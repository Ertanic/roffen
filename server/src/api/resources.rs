use crate::{
    Response,
    api::ApiContext,
    utils::{make_bad_request, make_internal_error, make_json_response, make_not_found},
};
use futures_util::{StreamExt, future::BoxFuture};
use log::error;
use macros::callback;
use serde::Serialize;
use vfs::error::VfsErrorKind;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    File,
    Directory,
}

#[derive(Serialize)]
pub struct ResourceInfo {
    pub name: String,
    pub path: String,
    pub link: String,
    pub resource_type: ResourceType,
}

#[callback]
pub fn get_resources_in_folder(ctx: ApiContext) -> BoxFuture<'static, Response> {
    let path = ctx.query.get("path");
    if path.is_none() {
        return make_bad_request();
    }

    let folder = path.unwrap();
    let resources = match ctx.resources.read().await.get_resources_in_folder(folder).await {
        Ok(resources) => serde_json::to_string(&resources.collect::<Vec<_>>().await).unwrap(),
        Err(err) => {
            return match err.kind() {
                VfsErrorKind::FileNotFound => {
                    error!("{folder} is not found");
                    make_not_found()
                }
                VfsErrorKind::InvalidPath => {
                    error!("{folder} is not a folder");
                    make_bad_request()
                }
                _ => {
                    error!("unable to get resources in folder {folder} because {err}");
                    make_internal_error()
                }
            };
        }
    };

    make_json_response(&resources)
}
