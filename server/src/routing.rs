use crate::ResourceRefType;
use futures_util::stream;
use http_body_util::StreamBody;
use hyper::{
    Request,
    body::{Bytes, Frame, Incoming},
    service::Service,
};
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
        let router = Arc::clone(&self.router);

        let result = async move {
            let result = router.at(req.uri().path());
            let not_found: Response = Response::new(StreamBody::new(Box::pin(stream::once(async {
                Ok::<Frame<Bytes>, std::io::Error>(Frame::data(Bytes::from("not found")))
            }))));

            match result {
                Ok(result) => match result.value {
                    ResourceRefType::Content(content) => {
                        let content = content.clone();
                        let stream = stream::once(async { Ok::<Frame<Bytes>, std::io::Error>(Frame::data(content)) });
                        let stream: BoxStream = Box::pin(stream);
                        let body = StreamBody::new(stream);
                        Ok(Response::new(body))
                    }
                    ResourceRefType::File(path) => {
                        let file = tokio::fs::File::open(path).await?;
                        let stream = ReaderStream::new(file);
                        let stream: BoxStream = Box::pin(StreamBody::new(
                            stream.filter_map(|buf| if let Ok(buf) = buf { Some(Ok(Frame::data(buf))) } else { None }),
                        ));
                        let body = StreamBody::new(stream);
                        Ok(Response::new(body))
                    }
                },
                Err(_) => Ok(not_found),
            }
        };

        Box::pin(result)
    }
}
