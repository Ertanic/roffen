mod consts;
mod logs;
mod routing;
mod utils;

use crate::{
    consts::{AUTH_COOKIE_NAME, AUTH_EXP, AUTH_FILENAME, CONTENT_FOLDER, INDEX_FILENAME, PAGES_FOLDER, PUBLIC_FOLDER},
    logs::setup_logger,
    routing::{Bulldozer, MethodRouter, get, post},
    utils::{make_bad_request, make_internal_error, make_not_allowed, make_not_found, make_response, make_unauthorized},
};
use cookie::{Cookie, CookieJar, time::Duration};
use futures_util::{future::BoxFuture, stream};
use http_body_util::{BodyExt, Collected, StreamBody};
use hyper::{
    Error, Method, Request, StatusCode,
    body::{Bytes, Frame, Incoming},
    header::HeaderValue,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use log::{debug, error, info, trace, warn};
use matchit::Router;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fmt::{Debug, Formatter},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::net::TcpListener;
use url_encoded_data::UrlEncodedData;
use crate::utils::make_see_other;

type BoxStream = stream::BoxStream<'static, Result<Frame<Bytes>, std::io::Error>>;
type Response = hyper::Response<StreamBody<BoxStream>>;
type ApiCallback = Box<dyn Fn(Request<Incoming>, Arc<AppContext>, Arc<AuthContext>) -> BoxFuture<'static, Response> + Sync + Send>;

#[derive(Debug)]
struct AppContext {
    content_folder: PathBuf,
    public_folder: PathBuf,
    pages_folder: PathBuf,
    auth_file: PathBuf,
}

impl AppContext {
    pub fn new(root: &Path) -> Self {
        let content_folder = root.join(CONTENT_FOLDER);
        let public_folder = content_folder.join(PUBLIC_FOLDER);
        let pages_folder = content_folder.join(PAGES_FOLDER);
        let auth_file = content_folder.join(AUTH_FILENAME);

        Self {
            content_folder,
            public_folder,
            pages_folder,
            auth_file,
        }
    }
}

#[derive(Deserialize)]
struct AuthContext {
    secret: String,
    users: Vec<UserCredentials>,
}

#[derive(Serialize, Deserialize)]
struct JwtPayload {
    username: String,
    exp: usize,
}

#[derive(Deserialize, PartialEq)]
struct UserCredentials {
    login: String,
    password: String,
}

#[derive(Clone, Debug)]
struct PageLayout(Arc<PathBuf>);

impl PageLayout {
    pub fn new(path: PathBuf) -> Self {
        Self(Arc::new(path))
    }
}

struct PageDir {
    path: PathBuf,
    inherited_layouts: Vec<PageLayout>,
}

impl PageDir {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            inherited_layouts: vec![],
        }
    }
}

enum ResourceRefType {
    File(PathBuf),
    Content(Bytes),
    Page { index: PathBuf, layouts: Vec<PageLayout> },
    Api(ApiCallback),
}

impl Debug for ResourceRefType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceRefType::File(path) => write!(f, "{:?}", path),
            ResourceRefType::Content(content) => write!(f, "{}", String::from_utf8_lossy(content)),
            ResourceRefType::Page { index, layouts } => write!(f, "path: {index:?}, layouts: {layouts:?}"),
            ResourceRefType::Api(_) => write!(f, "api handler"),
        }
    }
}

#[tokio::main]
async fn main() {
    setup_logger().expect("unable to start logs");

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084);
    let listener = TcpListener::bind(addr).await.expect("failed to bind");

    info!("listening on {}", addr);

    let root = get_root();
    let context = Arc::new(AppContext::new(&root));
    let auth_context = Arc::new(load_auth(&context.auth_file).await);

    debug!("auth context loaded");
    debug!("root: {root:?}");
    debug!("app context: {context:#?}");

    let mut router = MethodRouter::default();
    router
        .add(post("/admin/login"), ResourceRefType::Api(Box::new(login)))
        .expect("failed to register /login route");
    let router = load_public(&context.public_folder, router).await;
    let router = load_pages(&context.pages_folder, router).await;

    let router = Arc::new(router);
    let bulldozer = Arc::new(Bulldozer::new(router, context, auth_context));

    loop {
        let req = listener.accept().await.expect("failed to accept client");
        let io = TokioIo::new(req.0);

        let bulldozer = bulldozer.clone();

        tokio::task::spawn(async move {
            if let Err(err) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, bulldozer)
                .await
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}

fn login(mut _req: Request<Incoming>, _ctx: Arc<AppContext>, auth: Arc<AuthContext>) -> BoxFuture<'static, Response> {
    let ctx = Arc::clone(&_ctx);

    let query = _req
        .uri()
        .query()
        .map(|q| {
            let mut query = HashMap::new();
            for (key, value) in q.split('&').map(|s| s.split_once('=').unwrap()) {
                query.insert(key.to_owned(), value.to_owned());
            }
            query
        })
        .unwrap_or_default();

    let cookies = _req
        .headers()
        .get_all("Cookie")
        .iter()
        .filter_map(|c| {
            let str = c.to_str().ok()?;
            let cookie = Cookie::parse(str).ok()?;
            Some((cookie.name().to_owned(), cookie.value().to_owned()))
        })
        .collect::<HashMap<_, _>>();

    let result = async move {
        if !cookies.is_empty()
            && let Some(jwt) = cookies.get(AUTH_COOKIE_NAME)
        {
            trace!("found jwt");
            let key = DecodingKey::from_secret(auth.secret.as_bytes());
            let result = match jsonwebtoken::decode::<JwtPayload>(jwt, &key, &Validation::default()) {
                Ok(t) => t,
                Err(err) => {
                    error!("failed to decode jwt because {err}");
                    return make_unauthorized();
                }
            };

            if result.claims.exp < jiff::Timestamp::now().as_second() as usize {
                let cookie = Cookie::build((AUTH_COOKIE_NAME, "")).max_age(Duration::seconds(0)).build();
                let val = HeaderValue::from_str(&cookie.to_string()).expect("unable to build header value");
                _req.headers_mut().insert("Set-Cookie", val);
                return make_see_other("/admin/login");
            }
        }

        if _req.headers().get("Content-Type") != Some(&"application/x-www-form-urlencoded".parse().unwrap()) {
            return make_bad_request();
        }

        let body = match _req.into_body().collect().await {
            Ok(b) => String::from_utf8_lossy(&b.to_bytes()).to_string(),
            Err(err) => {
                error!("failed to collect request body because {err}");
                return make_internal_error();
            }
        };

        let credentials = UrlEncodedData::parse_str(&body);
        let username = credentials
            .get("login")
            .map(|p| p.first().map(ToString::to_string).unwrap_or_default())
            .unwrap_or_default();
        let password = credentials
            .get("password")
            .map(|p| p.first().map(ToString::to_string).unwrap_or_default())
            .unwrap_or_default();
        let credentials = UserCredentials { login: username, password };

        for user in &auth.users {
            if *user != credentials {
                continue;
            }

            let now = jiff::Timestamp::now().as_second() as usize;
            let exp = now + AUTH_EXP;
            let payload = JwtPayload {
                username: user.login.to_string(),
                exp,
            };
            let encode_key = EncodingKey::from_secret(auth.secret.as_bytes());

            let jwt = jsonwebtoken::encode(&Header::default(), &payload, &encode_key).expect("failed to encode jwt");

            let cookie = Cookie::build((AUTH_COOKIE_NAME, jwt))
                .path("/admin")
                .http_only(true)
                .max_age(Duration::seconds(exp as i64))
                .build();

            let mut response = if let Some(next) = query.get("next") {
                make_see_other(next)
            }
            else {
                make_response(StatusCode::OK, "authorized")
            };

            let val = HeaderValue::from_str(&cookie.to_string()).expect("unable to build header value");
            response.headers_mut().insert("Set-Cookie", val);

            return response;
        }

        make_not_found()
    };

    Box::pin(result)
}

async fn load_auth(auth_config_path: &Path) -> AuthContext {
    let auth_context_content = tokio::fs::read_to_string(&auth_config_path)
        .await
        .expect("unable to read auth config file");
    toml::from_str(&auth_context_content).expect("unable to parse auth config file")
}

async fn load_pages(pages_root: &Path, mut router: MethodRouter) -> MethodRouter {
    let mut pages = vec![PageDir::new(pages_root.to_path_buf())];

    while let Some(page) = pages.pop() {
        let mut children = vec![];
        let mut index_file = None;
        let mut layouts = vec![];

        let mut page_reader = match tokio::fs::read_dir(&page.path).await {
            Ok(r) => r,
            Err(err) => {
                warn!("unable to read folder {:?} because {err}, skipping", page.path);
                continue;
            }
        };

        while let Ok(Some(entry)) = page_reader.next_entry().await {
            let filetype = match entry.file_type().await {
                Ok(t) => t,
                Err(err) => {
                    warn!("unable to get file type {:?} because {err}, skipping", entry.path());
                    continue;
                }
            };

            if filetype.is_dir() {
                children.push(PageDir::new(entry.path()));
            }
            else {
                let filename = entry.file_name().to_string_lossy().to_string();
                if filename == INDEX_FILENAME {
                    index_file = Some(entry.path());
                }
                else {
                    let layout = PageLayout::new(entry.path());
                    layouts.push(layout);
                }
            }
        }

        while let Some(mut child) = children.pop() {
            child.inherited_layouts.extend(layouts.iter().cloned());
            pages.push(child);
        }

        if index_file.is_none() {
            warn!("{INDEX_FILENAME} not found in page folder {:?}", page.path);
            continue;
        }

        let name = page
            .path
            .strip_prefix(pages_root)
            .expect("failed to strip prefix")
            .to_string_lossy()
            .to_string();
        let normalized = normalize_route(&name);

        layouts.extend(page.inherited_layouts);

        let resource = ResourceRefType::Page {
            index: index_file.unwrap(),
            layouts,
        };

        match router.add(get(&normalized), resource) {
            Ok(_) => {
                debug!("page route {normalized} has been registered")
            }
            Err(err) => {
                warn!("page route {normalized} is not registered because {err}");
            }
        }
    }

    router
}

async fn load_public(public_root: &Path, mut router: MethodRouter) -> MethodRouter {
    let mut dirs = vec![public_root.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        let mut folder_reader = match tokio::fs::read_dir(&dir).await {
            Ok(reader) => reader,
            Err(err) => {
                warn!("unable to read folder {dir:?} because {err}, skipping");
                continue;
            }
        };

        while let Ok(Some(entry)) = folder_reader.next_entry().await {
            let filetype = match entry.file_type().await {
                Ok(t) => t,
                Err(err) => {
                    warn!("unable to get file type of {:?} because {err}, skipping", entry.path());
                    continue;
                }
            };

            if filetype.is_dir() {
                dirs.push(entry.path());
            }
            else {
                let name = entry
                    .path()
                    .strip_prefix(public_root)
                    .expect("failed to strip prefix")
                    .to_string_lossy()
                    .to_string();
                let normalized = normalize_route(&name);

                match router.add(get(&normalized), ResourceRefType::File(entry.path())) {
                    Ok(_) => {
                        debug!("route \"{normalized}\" has been registered");
                    }
                    Err(err) => {
                        warn!("route \"{normalized}\" is not registered because {err}");
                    }
                }
            }
        }
    }

    router
}

fn normalize_route(route: &str) -> String {
    if route.starts_with("/") {
        route.to_string()
    }
    else {
        "/".to_owned() + route
    }
    .replace('\\', "/")
}

fn get_root() -> PathBuf {
    let app_path = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| env::current_exe())
        .expect("unable to get app path");
    app_path.parent().expect("no app parent dir").to_path_buf()
}
