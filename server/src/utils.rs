use crate::Response;
use futures_util::{TryFutureExt, stream};
use http_body_util::StreamBody;
use hyper::{
    Method, StatusCode,
    body::{Bytes, Frame},
};

pub fn make_see_other(url: &str) -> Response {
    let mut response = make_empty_response(StatusCode::SEE_OTHER);
    response
        .headers_mut()
        .insert("Location", url.parse().unwrap_or_else(|_| "/".parse().unwrap()));
    response
}

pub fn make_unauthorized() -> Response {
    make_response(StatusCode::UNAUTHORIZED, "unauthorized")
}

pub fn make_not_found() -> Response {
    make_response(StatusCode::NOT_FOUND, "not found")
}

pub fn make_bad_request() -> Response {
    make_response(StatusCode::BAD_REQUEST, "bad request")
}

pub fn make_not_allowed(method: Method) -> Response {
    make_response(StatusCode::METHOD_NOT_ALLOWED, &format!("method {} not allow", method))
}

pub fn make_internal_error() -> Response {
    make_response(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
}

pub fn make_empty_response(status: StatusCode) -> Response {
    let mut res = Response::new(StreamBody::new(Box::pin(stream::empty())));
    *res.status_mut() = status;
    res
}

pub fn make_response(status: StatusCode, content: &str) -> Response {
    let content = content.to_owned();
    let mut res = Response::new(StreamBody::new(Box::pin(stream::once(async move {
        Ok::<Frame<Bytes>, std::io::Error>(Frame::data(Bytes::from(content)))
    }))));
    *res.status_mut() = status;
    res
}
