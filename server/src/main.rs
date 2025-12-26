mod consts;
mod routing;

use crate::{
    consts::{CONTENT_FOLDER, PUBLIC_FOLDER},
    routing::Bulldozer,
};
use hyper::body::Bytes;
use hyper_util::rt::{TokioExecutor, TokioIo};
use matchit::Router;
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::net::TcpListener;

enum ResourceRefType {
    File(PathBuf),
    Content(Bytes),
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084);
    let listener = TcpListener::bind(addr).await.expect("failed to bind");

    let mut router = Router::new();
    router
        .insert("/", ResourceRefType::Content(Bytes::from_static(b"Hello from roffen!")))
        .unwrap();

    let root = get_root();
    let content_folder = root.join(CONTENT_FOLDER);
    let public_folder = content_folder.join(PUBLIC_FOLDER);

    let public_folder = load_public(&public_folder).await;
    router.merge(public_folder).expect("unable to merge routes");

    let router = Arc::new(router);
    let bulldozer = Arc::new(Bulldozer::new(router));

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

async fn load_public(public_root: &Path) -> Router<ResourceRefType> {
    let mut dirs = vec![public_root.to_path_buf()];

    let mut routes = Router::new();
    while let Some(dir) = dirs.pop() {
        let mut folder_reader = tokio::fs::read_dir(dir).await.expect("failed to read dir");
        while let Ok(Some(entry)) = folder_reader.next_entry().await {
            let filetype = entry.file_type().await.expect("failed to get file type");
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

                routes
                    .insert(normalized, ResourceRefType::File(entry.path()))
                    .expect("unable to add route");
            }
        }
    }

    routes
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