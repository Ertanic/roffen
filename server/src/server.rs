use crate::{
    AppContext, Response,
    consts::{CERTS_FOLDER, DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT},
    resources::SecurityContext,
    routing::{Bulldozer, HookContext, MethodRouter},
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use log::{error, info};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    },
};

pub type ArcBulldozer = Arc<Bulldozer>;

pub struct Server {
    bulldozer: ArcBulldozer,
    addr: String,
    sec: Option<SecurityContext>,
    root: PathBuf,
}

impl Server {
    pub fn new(root: PathBuf, addr: String, app_context: Arc<AppContext>) -> Self {
        let bulldozer = Arc::new(Bulldozer::new(Arc::clone(&app_context)));
        Self {
            bulldozer,
            sec: None,
            root,
            addr,
        }
    }

    pub async fn routes(&mut self, builder: impl FnOnce(&mut MethodRouter)) -> &mut Self {
        self.bulldozer.routes(builder).await;
        self
    }

    pub async fn load_public(&mut self) -> &mut Self {
        self.bulldozer.load_public().await;
        self
    }

    pub async fn load_pages(&mut self) -> &mut Self {
        self.bulldozer.load_pages().await;
        self
    }

    pub async fn add_hook(&mut self, hook: impl Fn(&HookContext) -> Option<Response> + 'static + Send + Sync + Clone) -> &mut Self {
        self.bulldozer.add_hook(Box::new(hook)).await;
        self
    }

    pub fn set_security(&mut self, sec: Option<SecurityContext>) -> &mut Self {
        self.sec = sec;
        self
    }

    pub async fn serve(&self) {
        let port = if self.sec.is_some() { DEFAULT_HTTPS_PORT } else { DEFAULT_HTTP_PORT };
        let addr: SocketAddr = format!("{}:{port}", self.addr).parse().expect("invalid address");

        let listener = TcpListener::bind(addr).await.expect("failed to bind");

        if let Some(sec) = &self.sec {
            let certs_folder = self.root.join(CERTS_FOLDER);
            let cert_path = if sec.tls.cert.is_absolute() {
                sec.tls.cert.clone()
            }
            else {
                certs_folder.join(&sec.tls.cert)
            };

            let cert_key_path = if sec.tls.key.is_absolute() {
                sec.tls.key.clone()
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
            info!("listening on https://{}", addr);

            loop {
                let tcp_stream = match listener.accept().await {
                    Ok((stream, _)) => stream,
                    Err(err) => {
                        error!("failed to accept client: {err:#}");
                        continue;
                    }
                };

                let tls_acceptor = tls_acceptor.clone();
                let bulldozer = self.bulldozer.clone();

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
            info!("listening on http://{}", addr);
            loop {
                let (tcp_stream, _) = listener.accept().await.expect("failed to accept client");
                let io = TokioIo::new(tcp_stream);

                let bulldozer = self.bulldozer.clone();

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
}
