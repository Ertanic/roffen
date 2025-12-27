use crate::ResourceRefType;
use futures_util::stream;
use http_body_util::StreamBody;
use hyper::{
    Request, StatusCode,
    body::{Bytes, Frame, Incoming},
    http::HeaderValue,
    service::Service,
};
use log::{error, trace, warn};
use matchit::Router;
use std::{pin::Pin, sync::Arc};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

type BoxStream = stream::BoxStream<'static, Result<Frame<Bytes>, std::io::Error>>;
type Response = hyper::Response<StreamBody<BoxStream>>;

pub struct Bulldozer {
    router: Arc<Router<ResourceRefType>>,
}

impl Bulldozer {
    pub fn new(router: Arc<Router<ResourceRefType>>) -> Self {
        Self { router }
    }
}

impl Service<Request<Incoming>> for Bulldozer {
    type Response = Response;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        trace!("getting request {} \"{}\"", req.method(), req.uri().path());

        let router = Arc::clone(&self.router);

        let result = async move {
            let result = router.at(req.uri().path());
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
                        let rendered = template
                            .render(&engine, upon::value! {})
                            .to_string()
                            .unwrap_or_else(|_| panic!("unable to render {index:?}"));

                        let stream = stream::once(async { Ok::<Frame<Bytes>, std::io::Error>(Frame::data(Bytes::from(rendered))) });
                        let stream: BoxStream = Box::pin(stream);
                        let body = StreamBody::new(stream);
                        Ok(Response::new(body))
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
