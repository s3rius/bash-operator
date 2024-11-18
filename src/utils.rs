use std::str::FromStr;

use kube::{api::GroupVersionKind, core::GroupVersion};

use crate::error::BOResult;

pub fn get_gvk(gvk: &str) -> BOResult<GroupVersionKind> {
    let split = gvk.split('/');
    let mut a = split.collect::<Vec<_>>();
    let kind = a.pop().unwrap();
    let gv = GroupVersion::from_str(&a.join("/"))?;
    Ok(GroupVersionKind::gvk(&gv.group, &gv.version, kind))
}
