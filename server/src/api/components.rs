use crate::{
    Response,
    api::ApiContext,
    utils::{make_bad_request, make_internal_error, make_js_response, make_json_response, make_not_found},
};
use futures_util::future::BoxFuture;
use knus::Decode;
use log::error;
use macros::callback;
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::str::FromStr;
use tokio_stream::StreamExt;

#[derive(Decode, Serialize)]
pub struct HtmlAttr {
    #[knus(argument, str)]
    pub name: String,
    #[knus(argument)]
    pub value: String,
}

#[derive(Decode, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
pub enum HtmlType {
    Content(#[knus(argument)] String),
    Html(Html),
}

#[skip_serializing_none]
#[derive(Decode, Serialize, Default)]
pub struct Html {
    #[knus(argument)]
    pub element: String,
    #[knus(children(name = "attr"))]
    pub attrs: Option<Vec<HtmlAttr>>,
    #[knus(child, unwrap(children))]
    pub children: Option<Vec<HtmlType>>,
}

#[derive(Decode, Serialize, Default)]
pub enum ComponentPropertyType {
    #[default]
    String,
    Number,
    Boolean,
}

impl FromStr for ComponentPropertyType {
    type Err = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(ComponentPropertyType::String),
            "number" => Ok(ComponentPropertyType::Number),
            "boolean" => Ok(ComponentPropertyType::Boolean),
            _ => Err("invalid type".into()),
        }
    }
}

#[skip_serializing_none]
#[derive(Decode, Serialize, Default)]
pub struct ComponentProperty {
    #[knus(argument)]
    pub name: String,
    #[knus(child, unwrap(argument))]
    pub lang_key: String,
    #[knus(child, unwrap(argument))]
    pub default: Option<String>,
    #[knus(child, unwrap(argument))]
    pub max: Option<String>,
    #[knus(child, unwrap(argument))]
    pub min: Option<String>,
    #[knus(type_name)]
    pub type_name: Option<ComponentPropertyType>,
}

#[skip_serializing_none]
#[derive(Decode, Serialize, Default)]
pub struct ComponentContainerProperties {
    #[knus(child, unwrap(argument))]
    pub row: Option<String>,
    #[knus(child, unwrap(argument))]
    pub col: Option<String>,
    #[knus(child, unwrap(argument))]
    pub classes: Option<String>,
}

#[skip_serializing_none]
#[derive(Decode, Serialize, Default)]
pub struct Component {
    #[knus(argument)]
    pub name: String,
    #[knus(child, unwrap(argument))]
    pub lang_key: String,
    #[knus(child)]
    pub html: Html,
    #[knus(child)]
    pub container: Option<ComponentContainerProperties>,
    #[knus(children(name = "property"))]
    pub properties: Vec<ComponentProperty>,
}

#[derive(Decode)]
pub struct Document {
    #[knus(child)]
    pub component: Component,
}

#[derive(Serialize)]
pub struct ComponentProperties {
    pub name: String,
    pub properties: Vec<ComponentProperty>,
    pub container: Option<ComponentContainerProperties>,
    pub html: Html,
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
        Ok(comps) => comps.filter(|c| c.name == comp_name_param).collect::<Vec<_>>().await,
        Err(err) => {
            error!("unable to load components list because {err}");
            return make_internal_error();
        }
    };

    if let Some(comp) = comps.first() {
        const JS: &str = "console.log('todo');";
        make_js_response(JS)
    }
    else {
        make_not_found()
    }
}

#[callback]
pub fn get_components(ctx: ApiContext) -> BoxFuture<'static, Response> {
    let resources = ctx.resources.read().await;
    let comps = match resources.load_components().await {
        Ok(comps) => {
            comps
                .map(|c| ComponentProperties {
                    name: c.name,
                    html: c.html,
                    container: c.container,
                    properties: c.properties,
                })
                .collect::<Vec<_>>()
                .await
        }
        Err(err) => {
            error!("unable to load components list because {err}");
            return make_internal_error();
        }
    };

    let content = serde_json::to_string(&comps).unwrap();

    make_json_response(content.as_str())
}
