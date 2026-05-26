use crate::{api::auth::JwtPayload, config::ArcConfig, resources::ResourceManager};
use hyper::{Request, body::Incoming};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub mod auth;
pub mod components;
pub mod posts;
pub mod resources;

pub struct ApiContext {
    pub params: HashMap<String, String>,
    pub request: Request<Incoming>,
    pub query: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub config: ArcConfig,
    pub resources: Arc<RwLock<ResourceManager>>,
    pub jwt: Option<JwtPayload>,
}
