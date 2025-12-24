use http_body_util::Full;
use hyper::{
    Request, Response, StatusCode,
    body::{Bytes, Incoming},
    service::Service,
};
use matchit::Router;
use std::pin::Pin;

pub struct Bulldozer {
    router: Router<&'static str>,
}

impl Bulldozer {
    pub fn new(router: Router<&'static str>) -> Self {
        Self { router }
    }
}

impl Service<Request<Incoming>> for Bulldozer {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let result = self.router.at(req.uri().path());

        let result = match result {
            Ok(result) => Response::builder()
                .body(Full::new(Bytes::from_static(result.value.as_bytes())))
                .expect("unable to build response"),
            Err(_) => Response::builder().status(StatusCode::NOT_FOUND).body(Full::new(Bytes::from_static("not found".as_bytes()))).unwrap(),
        };

        Box::pin(async { Ok(result) })
    }
}
