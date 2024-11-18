use std::{
    io::{BufRead, BufReader, BufWriter, Seek, Write},
    sync::Arc,
    time::Duration,
};

use futures::StreamExt;
use kube::{
    api::{ApiResource, DynamicObject, PatchParams},
    runtime::{controller::Action, watcher, Controller},
    Api, Resource, ResourceExt,
};

use crate::{
    cli_args::Cli,
    error::{BOError, BOResult},
    utils::get_gvk,
};

pub struct Context {
    pub args: Cli,
    pub api: kube::Api<DynamicObject>,
    pub api_resource: ApiResource,
}

pub async fn run_operator(args: Cli) -> BOResult<()> {
    let gvk = get_gvk(&args.gvk)?;
    tracing::debug!("GVK parsed successfully");
    let client = kube::Client::try_default().await?;
    let (api_resource, _caps) = kube::discovery::pinned_kind(&client, &gvk).await?;
    tracing::debug!("Got APIResource {:?}", api_resource);
    let api = if args.all_namespaces {
        Api::<DynamicObject>::all_with(client, &api_resource)
    } else {
        let namespace = args
            .namespace
            .as_ref()
            .map(String::as_str)
            .unwrap_or(client.default_namespace());
        Api::<DynamicObject>::namespaced_with(client.clone(), namespace, &api_resource)
    };
    let wc = watcher::Config::default();

    let context = Arc::new(Context {
        args,
        api: api.clone(),
        api_resource: api_resource.clone(),
    });
    Controller::new_with(api, wc, api_resource)
        .run(reconcile, on_error, context)
        .for_each(|res| async move {
            match res {
                Ok((obj_ref, _)) => {
                    tracing::info!("reconciled {:?}", obj_ref.name);
                }
                Err(err) => match err {
                    kube::runtime::controller::Error::ObjectNotFound(_) => {
                        tracing::debug!("Object not found");
                    }
                    _ => {
                        tracing::error!("Couldn't reconcile an object. Reason: {}", err);
                    }
                },
            }
        })
        .await;
    Ok(())
}

async fn reconcile(obj: Arc<DynamicObject>, ctx: Arc<Context>) -> BOResult<Action> {
    let mut action = "Apply";
    if obj.meta().deletion_timestamp.is_some() {
        action = "Delete";
    }
    let ret_code = run_function(action, &obj, &ctx).await?;
    Ok(Action::requeue(Duration::from_secs(ret_code as u64)))
}

async fn run_function(action: &str, obj: &DynamicObject, ctx: &Arc<Context>) -> BOResult<u64> {
    let mut file_writer = BufWriter::new(tempfile::NamedTempFile::new()?);
    let mut to_dump = serde_yaml::to_value(&obj)?;
    let raw_obj = to_dump.as_mapping_mut().unwrap();
    raw_obj.insert(
        serde_yaml::Value::String("apiVersion".to_string()),
        serde_yaml::Value::String(ctx.api_resource.api_version.clone()),
    );
    raw_obj.insert(
        serde_yaml::Value::String("kind".to_string()),
        serde_yaml::Value::String(ctx.api_resource.kind.clone()),
    );
    write!(file_writer, "{}", serde_yaml::to_string(&to_dump)?)?;
    file_writer.flush()?;
    let mut manifest = file_writer.into_inner()?;
    let manifest_path = manifest.path().to_str().unwrap();
    let requeue_file = tempfile::NamedTempFile::new()?;
    tracing::debug!("Manifest file path: {}", manifest_path);
    let ret = tokio::process::Command::new("bash")
        .arg("-c")
        .arg(format!(
            "{} '{}' '{}' '{}'",
            &ctx.args.func_name,
            action,
            &manifest_path,
            &requeue_file.path().to_str().unwrap()
        ))
        .spawn()?
        .wait()
        .await?;
    if !ret.success() {
        return Err(BOError::ErrorStatusCode(ret.code().unwrap_or(1)));
    }
    manifest.seek(std::io::SeekFrom::Start(0))?;
    let new_manifest: serde_yaml::Value = serde_yaml::from_reader(BufReader::new(manifest))?;
    if new_manifest != to_dump {
        tracing::info!("Changes detected. Updating the object.");
        let patch = kube::api::Patch::Merge(serde_json::json!(new_manifest));
        let name = obj.name_any();
        ctx.api
            .patch(&name, &PatchParams::default(), &patch)
            .await?;
    }
    let mut requeue_duration_str = String::new();
    BufReader::new(requeue_file).read_line(&mut requeue_duration_str)?;
    return Ok(requeue_duration_str.trim().parse().unwrap_or(10));
}

fn on_error(_: Arc<DynamicObject>, _: &BOError, _context: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(10))
}
