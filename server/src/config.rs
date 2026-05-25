use crate::{
    api::auth::{AuthContext, UserCredentials},
    consts::DEFAULT_LANG_CODE,
    resources::SecurityContext,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type ArcConfig = Arc<RwLock<Config>>;

fn default_lang() -> String {
    DEFAULT_LANG_CODE.to_owned()
}

#[derive(Deserialize)]
pub struct LangConfig {
    #[serde(default = "default_lang")]
    pub current: String,
    #[serde(default = "default_lang")]
    pub default: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub auth: AuthContext,
    pub users: Vec<UserCredentials>,
    pub lang: LangConfig,
    #[serde(flatten)]
    pub sec: Option<SecurityContext>,
}
