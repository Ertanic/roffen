use crate::api::auth::{AuthContext, JwtPayload};
use hyper::{Request, body::Incoming};
use std::{collections::HashMap, sync::Arc};
use matchit::Params;
use tokio::sync::RwLock;
use crate::resources::ResourceManager;

pub mod auth;
pub mod posts;
pub mod components;

pub struct ApiContext<'a> {
    pub params: Params<'a, 'a>,
    pub request: Request<Incoming>,
    pub query: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub auth_context: Arc<AuthContext>,
    pub resources: Arc<RwLock<ResourceManager>>,
    pub jwt: Option<JwtPayload>,
}
