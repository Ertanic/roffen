use crate::{AppContext, AuthContext, BoxStream, JwtPayload, ResourceRefType, Response, consts::AUTH_COOKIE_NAME, utils::make_unauthorized};
use cookie::Cookie;
use futures_util::stream;
use http_body_util::StreamBody;
use hyper::{
    Method, Request, StatusCode,
    body::{Bytes, Frame, Incoming},
    http::HeaderValue,
    service::Service,
};
use jsonwebtoken::{DecodingKey, Validation};
use log::{debug, error, trace, warn};
use matchit::{Match, Router};
use std::{collections::HashMap, pin::Pin, sync::Arc};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

pub fn get(path: &str) -> MethodRoute<'_> {
    MethodRoute::new(Method::GET, path)
}

pub fn post(path: &str) -> MethodRoute<'_> {
    MethodRoute::new(Method::POST, path)
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
}

impl MethodRouter {
    pub fn add(&mut self, route: MethodRoute, resource: ResourceRefType) -> Result<(), matchit::InsertError> {
        match route.method {
            Method::GET => self.get.insert(route.path, resource),
            Method::POST => self.post.insert(route.path, resource),
            _ => Err(matchit::InsertError::Conflict {
                with: format!("no {} method router", route.method),
            }),
        }
    }

    pub fn at<'a>(&'a self, route: MethodRoute<'a>) -> Result<Match<'a, 'a, &'a ResourceRefType>, matchit::MatchError> {
        match route.method {
            Method::GET => self.get.at(route.path),
            Method::POST => self.post.at(route.path),
            _ => Err(matchit::MatchError::NotFound),
        }
    }
}

pub struct Bulldozer {
    router: Arc<MethodRouter>,
    context: Arc<AppContext>,
    auth: Arc<AuthContext>,
}

impl Bulldozer {
    pub fn new(router: Arc<MethodRouter>, context: Arc<AppContext>, auth: Arc<AuthContext>) -> Self {
        Self { router, context, auth }
    }
}

impl Service<Request<Incoming>> for Bulldozer {
    type Response = Response;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        trace!("getting request {} \"{}\"", req.method(), req.uri().path());

        let router = Arc::clone(&self.router);
        let ctx = Arc::clone(&self.context);
        let auth_context = Arc::clone(&self.auth);
        let method = req.method().clone();
        let path = req.uri().path().to_owned();

        let cookies = req
            .headers()
            .get_all("Cookie")
            .iter()
            .filter_map(|c| {
                let str = c.to_str().ok()?;
                let cookie = Cookie::parse(str).ok()?;
                Some((cookie.name().to_owned(), cookie.value().to_owned()))
            })
            .collect::<HashMap<_, _>>();

        let auth = {
            let jwt = cookies.get(AUTH_COOKIE_NAME);
            if let Some(jwt) = jwt {
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
                    }
                    else {
                        Some(payload.claims)
                    }
                }
                else {
                    None
                }
            }
            else {
                None
            }
        };

        let result = async move {
            let result = router.at(MethodRoute::new(method, &path));
            let mut not_found: Response = Response::new(StreamBody::new(Box::pin(stream::once(async {
                Ok::<Frame<Bytes>, std::io::Error>(Frame::data(Bytes::from("not found")))
            }))));
            *not_found.status_mut() = StatusCode::NOT_FOUND;

            let mut internal_error: Response = Response::new(StreamBody::new(Box::pin(stream::once(async {
                Ok::<Frame<Bytes>, std::io::Error>(Frame::data(Bytes::from("internal error")))
            }))));
            *internal_error.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

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
                        let file = tokio::fs::File::open(&path).await?;
                        let stream = ReaderStream::new(file);
                        let stream: BoxStream = Box::pin(StreamBody::new(
                            stream.filter_map(|buf| if let Ok(buf) = buf { Some(Ok(Frame::data(buf))) } else { None }),
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

                        trace!("init template engine...");
                        let mut engine = upon::Engine::new();

                        for layout in layouts {
                            let content = tokio::fs::read_to_string(&*layout.0).await?;
                            let filename = layout.0.file_name().unwrap().to_string_lossy();
                            let filename_components = filename.split('.').collect::<Vec<_>>();
                            let name = filename_components[..filename_components.len() - 1].join(".");
                            match engine.add_template(name.clone(), content) {
                                Ok(_) => {
                                    trace!("layout {name} has been registered in template engine");
                                }
                                Err(err) => {
                                    warn!("failed to add layout {name} ({:?}) in template engine because {err}", layout.0);
                                }
                            }
                        }

                        let content = tokio::fs::read_to_string(index).await?;
                        let template = match engine.compile(&content) {
                            Ok(t) => t,
                            Err(err) => {
                                error!("failed to compile {index:?} because {err:#}");
                                return Ok(internal_error);
                            }
                        };
                        let rendered = match template.render(&engine, upon::value! { 
                            auth: auth
                        }).to_string() {
                            Ok(r) => r,
                            Err(err) => {
                                error!("failed to render {index:?} because {err:#}");
                                return Ok(internal_error);
                            }
                        };

                        let stream = stream::once(async { Ok::<Frame<Bytes>, std::io::Error>(Frame::data(Bytes::from(rendered))) });
                        let stream: BoxStream = Box::pin(stream);
                        let body = StreamBody::new(stream);
                        Ok(Response::new(body))
                    }
                    ResourceRefType::Api(func) => {
                        let result = func(req, ctx, auth_context).await;
                        Ok(result)
                    }
                },
                Err(_) => {
                    trace!("route not found, sending 404");
                    Ok(not_found)
                }
            }
        };

        Box::pin(result)
    }
}
