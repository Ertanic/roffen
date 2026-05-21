mod api;
mod consts;
mod logs;
mod resources;
mod routing;
mod templates;
mod utils;
mod vfs;
mod watcher;

use crate::{
    api::{
        ApiContext,
        auth::{AuthContext, login, logout},
        components::get_component_js,
        posts::{create_post, delete_post, get_posts, new_post, update_post},
        resources::get_resources_in_folder,
    },
    consts::CONTENT_FOLDER,
    logs::setup_logger,
    resources::{ResourceManager, get_root},
    routing::{Bulldozer, BulldozerContext, MethodRouter, SystemPath, delete, get, patch, post},
    vfs::{PageLayout, VfsPath, init_vfs},
    watcher::init_watcher,
};
use futures_util::{future::BoxFuture, stream};
use http_body_util::StreamBody;
use hyper::{
    Method,
    body::{Bytes, Frame},
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use log::{error, info};
use std::{
    fmt::{Debug, Formatter},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::{net::TcpListener, sync::RwLock};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    },
};

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
    router
        .add(post("/api/posts"), ResourceRefType::Api(Box::new(create_post)))
        .expect("failed to register post /api/posts route");
    router
        .add(delete("/api/posts"), ResourceRefType::Api(Box::new(delete_post)))
        .expect("failed to register delete /api/posts route");
    router
        .add(get("/api/posts"), ResourceRefType::Api(Box::new(get_posts)))
        .expect("failed to register get /api/posts route");
    router
        .add(patch("/api/posts"), ResourceRefType::Api(Box::new(update_post)))
        .expect("failed to register patch /api/posts route");
    router
        .add(get("/components/js/{comp}"), ResourceRefType::Api(Box::new(get_component_js)))
        .expect("failed to register get /api/components route");
    router
        .add(get("/api/resources"), ResourceRefType::Api(Box::new(get_resources_in_folder)))
        .expect("failed to register get /api/resources route");
    router
        .add(get("/admin/posts/new"), ResourceRefType::Api(Box::new(new_post)))
        .expect("failed to register get /admin/posts/new route");

    let router = resources.read().await.load_public(router).await;
    let router = resources.read().await.load_pages(router).await;

    let router = Arc::new(RwLock::new(router));

    let system_paths = vec![
        SystemPath {
            path: "/admin",
            method: Method::GET,
            auth_required: true,
        },
        SystemPath {
            path: "/admin/posts",
            method: Method::GET,
            auth_required: true,
        },
        SystemPath {
            path: "/admin/pages",
            method: Method::GET,
            auth_required: true,
        },
        SystemPath {
            path: "/admin/resources",
            method: Method::GET,
            auth_required: true,
        },
    ];

    let ctx = BulldozerContext {
        router: Arc::clone(&router),
        resources: Arc::clone(&resources),
        auth: auth_context,
        vfs,
        system_paths,
    };
    let bulldozer = Arc::new(Bulldozer::new(ctx));

    init_watcher(&content_folder, Arc::clone(&router), Arc::clone(&resources));

    let sec = resources.read().await.load_security().await;
    if let Some(sec) = sec {
        let certs_folder = root.join("certs");
        let cert_path = if sec.tls.cert.is_absolute() {
            sec.tls.cert
        }
        else {
            certs_folder.join(&sec.tls.cert)
        };

        let cert_key_path = if sec.tls.key.is_absolute() {
            sec.tls.key
        }
        else {
            certs_folder.join(&sec.tls.key)
        };

        let cert = CertificateDer::pem_file_iter(cert_path)
            .expect("unable to load certificate")
            .collect::<Result<Vec<_>, _>>()
            .expect("unable to parse cert");

        let key = PrivateKeyDer::from_pem_file(cert_key_path).expect("unable to parse private key");

        let tls_config = ServerConfig::builder().with_no_client_auth().with_single_cert(cert, key).unwrap();
        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

        info!("TLS is enabled");

        loop {
            let (tcp_stream, _) = listener.accept().await.expect("failed to accept client");
            let tls_acceptor = tls_acceptor.clone();

            let bulldozer = bulldozer.clone();

            tokio::task::spawn(async move {
                let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                    Ok(tls_stream) => tls_stream,
                    Err(err) => {
                        error!("failed to perform tls handshake: {err:#}");
                        return;
                    }
                };

                if let Err(err) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                    .serve_connection(TokioIo::new(tls_stream), bulldozer)
                    .await
                {
                    error!("error serving connection: {}", err);
                }
            });
        }
    }
    else {
        loop {
            let (tcp_stream, _) = listener.accept().await.expect("failed to accept client");
            let io = TokioIo::new(tcp_stream);

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
}
