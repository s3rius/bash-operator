use std::{io::BufReader, path::PathBuf, str::FromStr};

use kube::{
    api::{DynamicObject, GroupVersionKind, Patch, PatchParams},
    core::GroupVersion,
    discovery::Scope,
};
use serde_json::json;
use serde_yaml::Value;

use crate::error::{BOError, BOResult};

pub fn get_gvk(gvk: &str) -> BOResult<GroupVersionKind> {
    let split = gvk.split('/');
    let mut a = split.collect::<Vec<_>>();
    let kind = a.pop().unwrap();
    let gv = GroupVersion::from_str(&a.join("/"))?;
    Ok(GroupVersionKind::gvk(&gv.group, &gv.version, kind))
}
