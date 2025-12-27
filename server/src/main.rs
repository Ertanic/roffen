mod consts;
mod logs;
mod routing;

use crate::{
    consts::{CONTENT_FOLDER, INDEX_FILENAME, PAGES_FOLDER, PUBLIC_FOLDER},
    logs::setup_logger,
    routing::Bulldozer,
};
use hyper::body::Bytes;
use hyper_util::rt::{TokioExecutor, TokioIo};
use log::{debug, info, warn};
use matchit::Router;
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::net::TcpListener;

#[derive(Clone)]
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
}

#[tokio::main]
async fn main() {
    setup_logger().expect("unable to start logs");

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084);
    let listener = TcpListener::bind(addr).await.expect("failed to bind");

    info!("listening on {}", addr);

    let mut router = Router::new();
    router
        .insert("/", ResourceRefType::Content(Bytes::from_static(b"Hello from roffen!")))
        .unwrap();

    let root = get_root();
    let content_folder = root.join(CONTENT_FOLDER);
    let public_folder = content_folder.join(PUBLIC_FOLDER);
    let pages_folder = content_folder.join(PAGES_FOLDER);

    debug!("root: {root:?}");
    debug!("content folder: {content_folder:?}");
    debug!("public folder: {public_folder:?}");
    debug!("pages folder: {pages_folder:?}");

    let public_folder = load_public(&public_folder).await;
    router.merge(public_folder).expect("unable to merge routes");

    let pages_folder = load_pages(&pages_folder).await;
    router.merge(pages_folder).expect("unable to merge pages");

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

async fn load_pages(pages_root: &Path) -> Router<ResourceRefType> {
    let mut routes = Router::new();
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

        match routes.insert(normalized.clone(), resource) {
            Ok(_) => {
                debug!("page route {normalized} has been registered")
            }
            Err(err) => {
                warn!("page route {normalized} is not registered because {err}");
            }
        }
    }

    routes
}

async fn load_public(public_root: &Path) -> Router<ResourceRefType> {
    let mut dirs = vec![public_root.to_path_buf()];

    let mut routes = Router::new();
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

                match routes.insert(normalized.clone(), ResourceRefType::File(entry.path())) {
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
