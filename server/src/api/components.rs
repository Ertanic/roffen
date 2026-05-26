use crate::{
    Response,
    api::ApiContext,
    utils::{make_bad_request, make_internal_error, make_js_response, make_json_response, make_not_found},
    vfs::VfsPath,
};
use futures_util::future::BoxFuture;
use log::error;
use macros::callback;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio_stream::StreamExt;

#[derive(Serialize)]
pub struct Component {
    pub js: VfsPath,
    pub meta: ComponentMeta,
}

#[derive(Deserialize, Serialize)]
pub struct ComponentMeta {
    pub name: String,
    pub title: String,
    pub html: String,
    #[serde(default)]
    pub defaults: HashMap<String, String>,
    #[serde(default)]
    pub properties: Vec<ComponentProperty>,
}

#[derive(Deserialize, Serialize)]
pub struct ComponentProperty {
    pub name: String,
    pub label: String,
    pub input_type: InputType,
}

#[derive(Deserialize, Serialize)]
pub enum InputType {
    Textarea,
    Number { min: Option<usize>, max: Option<usize> },
}

#[callback]
pub fn get_component_js(ctx: ApiContext) -> BoxFuture<'static, Response> {
    let comp_name_param = ctx.params.get("comp").map(ToOwned::to_owned);

    if comp_name_param.is_none() {
        return make_bad_request();
    }

    let comp_name_param = comp_name_param.unwrap();
    let resources = ctx.resources.read().await;
    let comps = match resources.load_components().await {
        Ok(comps) => comps.filter(|c| c.meta.name == comp_name_param).collect::<Vec<_>>().await,
        Err(err) => {
            error!("unable to load components list because {err}");
            return make_internal_error();
        }
    };

    if let Some(comp) = comps.first() {
        let js = match resources.load_component_js(comp).await {
            Ok(js) => js,
            Err(err) => {
                error!("unable to read js of component {} because {err}", comp.meta.name);
                return make_internal_error();
            }
        };

        make_js_response(&js)
    }
    else {
        make_not_found()
    }
}

#[callback]
pub fn get_components(ctx: ApiContext) -> BoxFuture<'static, Response> {
    let resources = ctx.resources.read().await;
    let comps = match resources.load_components().await {
        Ok(comps) => comps.map(|c| c.meta).collect::<Vec<_>>().await,
        Err(err) => {
            error!("unable to load components list because {err}");
            return make_internal_error();
        }
    };

    let content = serde_json::to_string(&comps).unwrap();

    make_json_response(content.as_str())
}