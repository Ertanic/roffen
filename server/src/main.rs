mod api;
mod consts;
mod logs;
mod resources;
mod routing;
mod utils;
mod vfs;
mod watcher;

use crate::{
    api::{
        ApiContext,
        auth::{AuthContext, login, logout},
    },
    consts::CONTENT_FOLDER,
    logs::setup_logger,
    resources::{ResourceManager, get_root},
    routing::{Bulldozer, MethodRouter, get, post},
    vfs::{PageLayout, VfsPath, init_vfs},
    watcher::init_watcher,
};
use futures_util::{future::BoxFuture, stream};
use http_body_util::StreamBody;
use hyper::body::{Bytes, Frame};
use hyper_util::rt::{TokioExecutor, TokioIo};
use log::info;
use std::{
    fmt::{Debug, Formatter},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::{net::TcpListener, sync::RwLock};

type BoxStream = stream::BoxStream<'static, Result<Frame<Bytes>, std::io::Error>>;
type Response = hyper::Response<StreamBody<BoxStream>>;
type ApiCallback = Box<dyn Fn(ApiContext) -> BoxFuture<'static, Response> + Sync + Send>;

enum ResourceRefType {
    File(VfsPath),
    Content(Bytes),
    Page { index: VfsPath, layouts: Vec<PageLayout> },
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
    let content_folder = root.join(CONTENT_FOLDER);
    let vfs = init_vfs(&content_folder).await;

    let resources = Arc::new(RwLock::new(ResourceManager::new(Arc::clone(&vfs))));

    let auth_context = Arc::new(resources.read().await.load_auth().await);

    let mut router = MethodRouter::default();
    router
        .add(post("/admin/login"), ResourceRefType::Api(Box::new(login)))
        .expect("failed to register /login route");
    router
        .add(get("/admin/logout"), ResourceRefType::Api(Box::new(logout)))
        .expect("failed to register /logout route");
    router
        .add(get("/health"), ResourceRefType::Content(Bytes::from("ok")))
        .expect("failed to register /health route");
    let router = resources.read().await.load_public(router).await;
    let router = resources.read().await.load_pages(router).await;

    let router = Arc::new(RwLock::new(router));
    let bulldozer = Arc::new(Bulldozer::new(Arc::clone(&router), auth_context, vfs));

    init_watcher(&content_folder, Arc::clone(&router), Arc::clone(&resources));

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
