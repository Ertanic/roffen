mod api;
mod consts;
mod logs;
mod resources;
mod routing;
mod utils;

use crate::{
    api::{
        ApiContext,
        auth::{AuthContext, login},
    },
    consts::{AUTH_FILENAME, CONTENT_FOLDER, PAGES_FOLDER, PUBLIC_FOLDER},
    logs::setup_logger,
    resources::{get_root, load_auth, load_pages, load_public},
    routing::{Bulldozer, MethodRouter, get, post},
};
use futures_util::{future::BoxFuture, stream};
use http_body_util::StreamBody;
use hyper::body::{Bytes, Frame};
use hyper_util::rt::{TokioExecutor, TokioIo};
use log::{debug, info};
use std::{
    fmt::{Debug, Formatter},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::net::TcpListener;

type BoxStream = stream::BoxStream<'static, Result<Frame<Bytes>, std::io::Error>>;
type Response = hyper::Response<StreamBody<BoxStream>>;
type ApiCallback = Box<dyn Fn(ApiContext) -> BoxFuture<'static, Response> + Sync + Send>;

#[derive(Debug)]
struct AppContext {
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
            public_folder,
            pages_folder,
            auth_file,
        }
    }
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
    router
        .add(get("/health"), ResourceRefType::Content(Bytes::from("ok")))
        .expect("failed to register /health route");
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
