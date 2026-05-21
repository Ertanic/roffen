use crate::{
    AuthContext, BoxStream, ResourceRefType, Response,
    api::{ApiContext, auth::JwtPayload},
    consts::AUTH_COOKIE_NAME,
    resources::ResourceManager,
    templates::functions::register_functions,
    utils::{make_internal_error, make_not_found, make_see_other},
    vfs::VirtualFS,
};
use cookie::Cookie;
use futures_util::{AsyncReadExt, stream};
use http_body_util::StreamBody;
use hyper::{
    HeaderMap, Method, Request,
    body::{Bytes, Frame, Incoming},
    http::HeaderValue,
    service::Service,
};
use jsonwebtoken::{DecodingKey, Validation};
use log::{error, trace, warn};
use matchit::{Match, Router};
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;
use url_encoded_data::UrlEncodedData;
use vfs::async_vfs::AsyncFileSystem;

pub fn get(path: &str) -> MethodRoute<'_> {
    MethodRoute::new(Method::GET, path)
}

pub fn post(path: &str) -> MethodRoute<'_> {
    MethodRoute::new(Method::POST, path)
}

pub fn delete(path: &str) -> MethodRoute<'_> {
    MethodRoute::new(Method::DELETE, path)
}

pub fn patch(path: &str) -> MethodRoute<'_> {
    MethodRoute::new(Method::PATCH, path)
}

#[derive(Debug)]
pub struct MethodRoute<'a> {
    method: Method,
    path: &'a str,
}

impl<'a> MethodRoute<'a> {
    pub fn new(method: Method, path: &'a str) -> Self {
        Self { method, path }
    }
}

#[derive(Default)]
pub struct MethodRouter {
    get: Router<ResourceRefType>,
    post: Router<ResourceRefType>,
    delete: Router<ResourceRefType>,
    patch: Router<ResourceRefType>,
}

impl MethodRouter {
    pub fn add(&mut self, route: MethodRoute, resource: ResourceRefType) -> Result<(), matchit::InsertError> {
        match route.method {
            Method::GET => self.get.insert(route.path, resource),
            Method::POST => self.post.insert(route.path, resource),
            Method::DELETE => self.delete.insert(route.path, resource),
            Method::PATCH => self.patch.insert(route.path, resource),
            _ => Err(matchit::InsertError::Conflict {
                with: format!("no {} method router", route.method),
            }),
        }
    }

    pub fn at<'a>(&'a self, route: MethodRoute<'a>) -> Result<Match<'a, 'a, &'a ResourceRefType>, matchit::MatchError> {
        match route.method {
            Method::GET => self.get.at(route.path),
            Method::POST => self.post.at(route.path),
            Method::DELETE => self.delete.at(route.path),
            Method::PATCH => self.patch.at(route.path),
            _ => Err(matchit::MatchError::NotFound),
        }
    }

    pub fn remove(&mut self, route: MethodRoute) {
        let _ = match route.method {
            Method::GET => self.get.remove(route.path),
            Method::POST => self.post.remove(route.path),
            Method::DELETE => self.delete.remove(route.path),
            Method::PATCH => self.patch.remove(route.path),
            _ => None,
        };
    }
}

#[derive(Clone)]
pub struct SystemPath {
    pub path: &'static str,
    pub method: Method,
    pub auth_required: bool,
}

pub struct BulldozerContext {
    pub router: Arc<RwLock<MethodRouter>>,
    pub auth: Arc<AuthContext>,
    pub vfs: VirtualFS,
    pub resources: Arc<RwLock<ResourceManager>>,
    pub system_paths: Vec<SystemPath>,
}

pub struct Bulldozer {
    router: Arc<RwLock<MethodRouter>>,
    auth: Arc<AuthContext>,
    vfs: VirtualFS,
    resources: Arc<RwLock<ResourceManager>>,
    system_paths: Vec<SystemPath>,
}

impl Bulldozer {
    pub fn new(ctx: BulldozerContext) -> Self {
        Self {
            router: ctx.router,
            auth: ctx.auth,
            vfs: ctx.vfs,
            resources: ctx.resources,
            system_paths: ctx.system_paths,
        }
    }
}

impl Service<Request<Incoming>> for Bulldozer {
    type Response = Response;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        trace!(
            "getting request {} \"{}\"",
            req.method(),
            req.uri().path_and_query().map(|a| a.as_str()).unwrap_or_else(|| req.uri().path())
        );

        let resources = Arc::clone(&self.resources);
        let vfs = Arc::clone(&self.vfs);
        let router = Arc::clone(&self.router);
        let auth_context = Arc::clone(&self.auth);
        let method = req.method().clone();
        let uri = req.uri();
        let path = uri.path().to_owned();
        let query = uri
            .query()
            .map(|s| {
                UrlEncodedData::parse_str(s)
                    .iter()
                    .map(|p| (p.0.to_string(), p.1.to_string()))
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default();

        let system_paths = self.system_paths.clone();

        let result = async move {
            let path_clone = path.clone();
            let router = router.read().await;
            let result = router.at(MethodRoute::new(method.clone(), &path_clone));

            let headers = req.headers().clone();
            let auth_ctx = Arc::clone(&auth_context);
            let cookies = Arc::new(LazyLock::new(|| parse_cookies(headers)));
            let cookies_jwt = Arc::clone(&cookies);
            let auth = Arc::new(LazyLock::new(move || decode_jwt_from_cookies(&cookies_jwt, &auth_ctx)));

            match result {
                Ok(result) => match result.value {
                    ResourceRefType::Content(content) => {
                        trace!("found content resource ref");
                        let content = content.clone();
                        let stream = stream::once(async { Ok::<Frame<Bytes>, std::io::Error>(Frame::data(content)) });
                        let stream: BoxStream = Box::pin(stream);
                        let body = StreamBody::new(stream);
                        Ok(Response::new(body))
                    }
                    ResourceRefType::File(path) => {
                        trace!("found file resource ref: {path:?}");
                        let file = match vfs.open_file(path).await {
                            Ok(f) => f,
                            Err(err) => {
                                error!("failed to open {path} file because {err}");
                                return Ok(make_not_found());
                            }
                        };

                        // convert async-std `Read` to tokio `AsyncRead`
                        let file = tokio_util::compat::FuturesAsyncReadCompatExt::compat(file);

                        let reader = ReaderStream::new(file);
                        let stream: BoxStream = Box::pin(StreamBody::new(
                            reader.filter_map(|buf| if let Ok(buf) = buf { Some(Ok(Frame::data(buf))) } else { None }),
                        ));
                        let body = StreamBody::new(stream);
                        let mut response = Response::new(body);

                        if let Some(mime) = mime_guess2::from_path(path).first() {
                            response
                                .headers_mut()
                                .insert("Content-Type", HeaderValue::from_str(mime.as_ref()).expect("mime error"));
                        }

                        Ok(response)
                    }
                    ResourceRefType::Page { index, layouts } => {
                        trace!("found page resource ref: {index:?}");

                        for sys_path in &system_paths {
                            if sys_path.path == path && sys_path.method == method && (sys_path.auth_required && auth.is_none()) {
                                trace!("found system path, redirecting to login page");
                                return Ok(make_see_other(format!("/admin/login?next={path}").as_str()));
                            }
                        }

                        trace!("init template engine...");
                        let mut engine = upon::Engine::new();

                        for layout in layouts {
                            let mut content = String::new();

                            let mut file = match vfs.open_file(&layout).await {
                                Ok(f) => f,
                                Err(err) => {
                                    error!("failed to open {layout} file because {err}");
                                    continue;
                                }
                            };

                            if let Err(err) = file.read_to_string(&mut content).await {
                                error!("unable to read {layout} file because {err}");
                                continue;
                            }

                            let filename = layout.split('/').next_back().unwrap();
                            let filename_components = filename.split('.').collect::<Vec<_>>();
                            let name = filename_components[..filename_components.len() - 1].join(".");
                            match engine.add_template(name.clone(), content) {
                                Ok(_) => {
                                    trace!("layout {name} has been registered in template engine");
                                }
                                Err(err) => {
                                    warn!("failed to add layout {name} ({layout}) in template engine because {err}");
                                }
                            }
                        }

                        engine.add_function("auth", {
                            let auth = Arc::clone(&auth);
                            move || (**auth).as_ref().map(|auth| upon::to_value(auth).unwrap())
                        });

                        register_functions(&mut engine, resources);

                        let mut content = String::new();
                        let mut file = match vfs.open_file(index).await {
                            Ok(f) => f,
                            Err(err) => {
                                error!("failed to open {index} file because {err}");
                                return Ok(make_not_found());
                            }
                        };
                        if let Err(err) = file.read_to_string(&mut content).await {
                            error!("failed to read {index} file because {err}");
                            return Ok(make_internal_error());
                        }

                        let template = match engine.compile(&content) {
                            Ok(t) => t,
                            Err(err) => {
                                error!("failed to compile {index:?} because {err:#}");
                                return Ok(make_internal_error());
                            }
                        };

                        let rendered = match template
                            .render(
                                &engine,
                                upon::value! {
                                    query: query,
                                },
                            )
                            .to_string()
                        {
                            Ok(r) => r,
                            Err(err) => {
                                error!("failed to render {index:?} because {err:#}");
                                return Ok(make_internal_error());
                            }
                        };

                        let stream = stream::once(async { Ok::<Frame<Bytes>, std::io::Error>(Frame::data(Bytes::from(rendered))) });
                        let stream: BoxStream = Box::pin(stream);
                        let body = StreamBody::new(stream);
                        Ok(Response::new(body))
                    }
                    ResourceRefType::Api(callback) => {
                        let cookies = (**cookies).clone();
                        let jwt = (**auth).clone();
                        let params = result.params;
                        let ctx = ApiContext {
                            params,
                            request: req,
                            query,
                            cookies,
                            auth_context,
                            resources,
                            jwt,
                        };
                        let result = callback(ctx).await;
                        Ok(result)
                    }
                },
                Err(_) => {
                    trace!("route not found, sending 404");
                    Ok(make_not_found())
                }
            }
        };

        Box::pin(result)
    }
}

fn parse_cookies(headers: HeaderMap) -> HashMap<String, String> {
    headers
        .get_all("Cookie")
        .iter()
        .filter_map(|c| {
            let str = c.to_str().ok()?;
            let cookie = Cookie::parse(str).ok()?;
            Some((cookie.name().to_owned(), cookie.value().to_owned()))
        })
        .collect::<HashMap<_, _>>()
}

fn decode_jwt_from_cookies(cookies: &HashMap<String, String>, auth_context: &AuthContext) -> Option<JwtPayload> {
    if let Some(jwt) = cookies.get(AUTH_COOKIE_NAME) {
        let key = DecodingKey::from_secret(auth_context.secret.as_bytes());
        let payload = match jsonwebtoken::decode::<JwtPayload>(jwt, &key, &Validation::default()) {
            Ok(t) => Some(t),
            Err(err) => {
                error!("failed to decode jwt because {err}");
                None
            }
        };

        if let Some(payload) = payload {
            if payload.claims.exp < jiff::Timestamp::now().as_second() as usize {
                None
            } else {
                Some(payload.claims)
            }
        } else {
            None
        }
    } else {
        None
    }
}
