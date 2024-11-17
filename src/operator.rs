use std::{io::Write, process::Command, str::FromStr, sync::Arc, time::Duration};

use futures::StreamExt;
use kube::{
    api::{DynamicObject, GroupVersionKind},
    core::GroupVersion,
    runtime::{controller::Action, watcher, Controller},
    Api, Resource,
};

use crate::{cli_args::OperatorArgs, utils::get_gvk};

pub struct Context {
    pub args: OperatorArgs,
}

pub async fn run_operator(args: OperatorArgs) -> anyhow::Result<()> {
    let gvk = get_gvk(&args.gvk)?;
    let client = kube::Client::try_default().await?;
    let (ar, _caps) = kube::discovery::pinned_kind(&client, &gvk).await?;

    let api = Api::<DynamicObject>::all_with(client, &ar);
    let wc = watcher::Config::default();

    let context = Arc::new(Context { args });
    Controller::new_with(api, wc, ar)
        .run(reconcile, on_error, context)
        .for_each(|res| async move {
            match res {
                Ok((obj_ref, _)) => {
                    tracing::info!("reconciled {:?}", obj_ref.name);
                }
                Err(err) => tracing::error!("Couldn't reconcile an object. Reason: {}", err),
            }
        })
        .await;
    Ok(())
}

async fn reconcile(obj: Arc<DynamicObject>, ctx: Arc<Context>) -> std::io::Result<Action> {
    let mut action = "Apply";
    if obj.meta().deletion_timestamp.is_some() {
        action = "Delete";
    }
    let ret_code = run_function(action, &obj, &ctx).await?;
    if ret_code <= 0 {
        return Ok(Action::await_change());
    }
    Ok(Action::requeue(Duration::from_secs(ret_code as u64)))
}

async fn run_function(
    action: &str,
    obj: &DynamicObject,
    ctx: &Arc<Context>,
) -> std::io::Result<i32> {
    let mut file = tempfile::NamedTempFile::new()?;
    write!(file, "{}", serde_json::to_string_pretty(obj)?)?;
    file.flush()?;
    let file_path = file.path().to_str().unwrap();
    tracing::debug!("TMP file path: {}", file_path);
    let ret = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "{} '{}' '{}'",
            &ctx.args.func_name, action, &file_path
        ))
        .spawn()?
        .wait()?;
    return Ok(ret.code().unwrap_or(0));
}

fn on_error(_: Arc<DynamicObject>, _: &std::io::Error, _context: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(10))
}
