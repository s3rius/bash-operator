use std::{io::BufReader, path::PathBuf, str::FromStr};

use kube::{
    api::{DynamicObject, GroupVersionKind, Patch, PatchParams},
    core::GroupVersion,
    discovery::Scope,
};
use serde_json::json;

pub fn get_gvk(gvk: &str) -> anyhow::Result<GroupVersionKind> {
    let split = gvk.split('/');
    let mut a = split.collect::<Vec<_>>();
    let kind = a.pop().unwrap();
    let gv = GroupVersion::from_str(&a.join("/"))?;
    Ok(GroupVersionKind::gvk(&gv.group, &gv.version, kind))
}

pub async fn update_finalizer(
    finalizer_name: &str,
    manifest_path: &PathBuf,
    add: bool,
) -> anyhow::Result<()> {
    let rdr = BufReader::new(std::fs::File::open(manifest_path)?);
    let res: serde_yaml::Value = serde_yaml::from_reader(rdr)?;
    let Some(res_map) = res.as_mapping() else {
        anyhow::bail!("Cannot parse manifest file!");
    };
    println!("{:?}", res_map["metadata"]["hui"].as_str());
    let gv = GroupVersion::from_str(&res_map["apiVersion"].as_str().unwrap_or(""))?;
    let gvk = GroupVersionKind::gvk(
        &gv.group,
        &gv.version,
        res_map["kind"].as_str().unwrap_or(""),
    );
    let client = kube::Client::try_default().await?;
    let (ar, caps) = kube::discovery::pinned_kind(&client, &gvk).await?;
    let api = if caps.scope == Scope::Cluster {
        kube::Api::<DynamicObject>::all_with(client, &ar)
    } else {
        let default_namespace = client.default_namespace();
        kube::Api::<DynamicObject>::namespaced_with(
            client.clone(),
            &res_map["metadata"]["namespace"]
                .as_str()
                .unwrap_or(default_namespace),
            &ar,
        )
    };

    let finalizers_value = res_map["metadata"]["finalizers"]
        .as_sequence()
        .cloned()
        .unwrap_or(Vec::new());

    let mut finalizers = finalizers_value
        .iter()
        .map(serde_yaml::Value::as_str)
        .flatten()
        .filter(|item| *item != finalizer_name)
        .collect::<Vec<_>>();

    if add {
        finalizers.push(finalizer_name);
    }

    api.patch(
        res_map["metadata"]["name"].as_str().unwrap_or_default(),
        &PatchParams::default(),
        &Patch::Merge(json!({
            "metadata": {
                "finalizers": finalizers,
            }
        })),
    )
    .await?;

    Ok(())
}
