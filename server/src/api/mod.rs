use crate::api::auth::{AuthContext, JwtPayload};
use hyper::{Request, body::Incoming};
use std::{collections::HashMap, sync::Arc};

pub mod auth;

pub struct ApiContext {
    pub request: Request<Incoming>,
    pub query: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub auth_context: Arc<AuthContext>,
    pub jwt: Option<JwtPayload>,
}
