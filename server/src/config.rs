use crate::{
    api::auth::{AuthContext, UserCredentials},
    consts::{DEFAULT_ADDR, DEFAULT_LANG_CODE},
    resources::SecurityContext,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type ArcConfig = Arc<RwLock<Config>>;

fn default_lang() -> String {
    DEFAULT_LANG_CODE.to_owned()
}

fn default_addr() -> String {
    DEFAULT_ADDR.to_owned()
}

fn default_server() -> ServerConfig {
    ServerConfig {
        host: default_addr(),
        port: None,
    }
}

#[derive(Deserialize)]
pub struct LangConfig {
    #[serde(default = "default_lang")]
    pub current: String,
    #[serde(default = "default_lang")]
    pub default: String,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_addr")]
    pub host: String,
    pub port: Option<u16>,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    pub auth: AuthContext,
    pub users: Vec<UserCredentials>,
    pub lang: LangConfig,
    #[serde(flatten)]
    pub sec: Option<SecurityContext>,
}
