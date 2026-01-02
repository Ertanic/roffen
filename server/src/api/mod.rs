use crate::api::auth::{AuthContext, JwtPayload};
use hyper::{Request, body::Incoming};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use crate::resources::ResourceManager;

pub mod auth;
pub mod posts;

pub struct ApiContext {
    pub request: Request<Incoming>,
    pub query: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub auth_context: Arc<AuthContext>,
    pub resources: Arc<RwLock<ResourceManager>>,
    pub jwt: Option<JwtPayload>,
}
