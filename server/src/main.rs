mod api;
mod config;
mod consts;
mod lang;
mod logs;
mod resources;
mod routing;
mod server;
mod templates;
mod utils;
mod vfs;
mod watcher;

use crate::{
    api::{
        ApiContext,
        auth::{AuthContext, login, logout},
        components::{get_component_js, get_components},
        posts::{create_post, delete_post, get_posts, new_post, update_post},
        resources::get_resources_in_folder,
    },
    config::ArcConfig,
    consts::{CONFIG_FILENAME, CONTENT_FOLDER, LANG_FOLDER, PAGES_FOLDER, PUBLIC_FOLDER},
    lang::LangManager,
    logs::setup_logger,
    resources::{ResourceManager, api, get_root},
    routing::{MethodRouter, delete, get, patch, post},
    server::Server,
    utils::make_see_other,
    vfs::{PageLayout, VfsPath, VirtualFS, init_vfs},
    watcher::{ConfigEventFabric, FilesWatcher, LangEventFabric, PagesEventFabric, PublicEventFabric},
};
use futures_util::{future::BoxFuture, stream};
use http_body_util::StreamBody;
use hyper::body::{Bytes, Frame};
use log::debug;
use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};
use tokio::sync::RwLock;

type BoxStream = stream::BoxStream<'static, Result<Frame<Bytes>, std::io::Error>>;
type Response = hyper::Response<StreamBody<BoxStream>>;
type ApiCallback = Box<dyn Fn(ApiContext) -> BoxFuture<'static, Response> + Sync + Send>;

struct AppContext {
    pub router: Arc<RwLock<MethodRouter>>,
    pub config: ArcConfig,
    pub vfs: VirtualFS,
    pub resources: Arc<RwLock<ResourceManager>>,
    pub lang_manager: LangManager,
}

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

    let root = get_root();
    let content_folder = root.join(CONTENT_FOLDER);
    let vfs = init_vfs(&content_folder).await;

    let resources = Arc::new(RwLock::new(ResourceManager::new(Arc::clone(&vfs))));
    let config = Arc::new(RwLock::new(resources.read().await.load_config().await));

    let lang_manager = {
        let config = config.read().await;
        debug!("current lang: {}, default: {}", config.lang.current, config.lang.default);
        resources
            .read()
            .await
            .load_langs(config.lang.default.clone(), config.lang.current.clone())
            .await
            .expect("unable to load languages")
    };

    let ctx = Arc::new(AppContext {
        router: Default::default(),
        resources: Arc::clone(&resources),
        config: Arc::clone(&config),
        vfs,
        lang_manager: lang_manager.clone(),
    });

    FilesWatcher::init(Arc::clone(&ctx), content_folder)
        .watch(PublicEventFabric, PUBLIC_FOLDER.as_ref())
        .watch(PagesEventFabric, PAGES_FOLDER.as_ref())
        .watch(LangEventFabric, LANG_FOLDER.as_ref())
        .watch(ConfigEventFabric, CONFIG_FILENAME.as_ref());

    let sec = config.read().await.sec.clone(); // avoiding deadlock

    Server::new(root, ctx)
        .add_hook(|ctx| {
            let path = ctx.request.uri().path();
            if path != "/admin/login" && path.starts_with("/admin") && ctx.jwt.is_none() {
                Some(make_see_other(&format!("/admin/login?next={path}")))
            }
            else {
                None
            }
        })
        .await
        .load_pages()
        .await
        .load_public()
        .await
        .routes(|r| {
            r.add(post("/admin/login"), api(login))
                .add(get("/admin/logout"), api(logout))
                .add(get("/health"), ResourceRefType::Content(Bytes::from("ok")))
                .add(post("/api/posts"), api(create_post))
                .add(delete("/api/posts"), api(delete_post))
                .add(get("/api/posts"), api(get_posts))
                .add(patch("/api/posts"), api(update_post))
                .add(get("/components/js/{comp}"), api(get_component_js))
                .add(get("/components"), api(get_components))
                .add(get("/api/resources"), api(get_resources_in_folder))
                .add(get("/admin/posts/new"), api(new_post));
        })
        .await
        .set_security(sec)
        .serve()
        .await;
}
