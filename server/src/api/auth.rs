use crate::{
    Response,
    api::ApiContext,
    consts::{AUTH_COOKIE_NAME, AUTH_EXP},
    utils::{make_bad_request, make_internal_error, make_not_found, make_response, make_see_other},
};
use cookie::{Cookie, time::Duration};
use futures_util::future::BoxFuture;
use http_body_util::BodyExt;
use hyper::{StatusCode, header::HeaderValue};
use jsonwebtoken::{EncodingKey, Header};
use log::{error, trace};
use macros::callback;
use serde::{Deserialize, Serialize};
use url_encoded_data::UrlEncodedData;

#[derive(Deserialize, Clone)]
pub struct AuthContext {
    pub secret: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JwtPayload {
    pub username: String,
    pub exp: usize,
}

#[derive(Deserialize, PartialEq, Clone)]
pub struct UserCredentials {
    login: String,
    password: String,
}

#[callback]
pub fn login(mut ctx: ApiContext) -> BoxFuture<'static, Response> {
    if let Some(jwt) = ctx.jwt {
        trace!("found jwt");
        if jwt.exp < jiff::Timestamp::now().as_second() as usize {
            let cookie = Cookie::build((AUTH_COOKIE_NAME, "_")).max_age(Duration::seconds(0)).build();
            let val = HeaderValue::from_str(&cookie.to_string()).expect("unable to build header value");
            ctx.request.headers_mut().insert("Set-Cookie", val);
            return make_see_other("/admin/login");
        }
    }

    if ctx.request.headers().get("Content-Type") != Some(&"application/x-www-form-urlencoded".parse().unwrap()) {
        return make_bad_request();
    }

    let body = match ctx.request.into_body().collect().await {
        Ok(b) => String::from_utf8_lossy(&b.to_bytes()).to_string(),
        Err(err) => {
            error!("failed to collect request body because {err}");
            return make_internal_error();
        }
    };

    let credentials = UrlEncodedData::parse_str(&body);
    let username = credentials
        .get("login")
        .map(|p| p.first().map(ToString::to_string).unwrap_or_default())
        .unwrap_or_default();
    let password = credentials
        .get("password")
        .map(|p| p.first().map(ToString::to_string).unwrap_or_default())
        .unwrap_or_default();
    let credentials = UserCredentials { login: username, password };

    for user in &ctx.config.read().await.users {
        if *user != credentials {
            continue;
        }

        let now = jiff::Timestamp::now().as_second() as usize;
        let exp = now + AUTH_EXP;
        let payload = JwtPayload {
            username: user.login.to_string(),
            exp,
        };
        let encode_key = EncodingKey::from_secret(ctx.config.read().await.auth.secret.as_bytes());

        let jwt = jsonwebtoken::encode(&Header::default(), &payload, &encode_key).expect("failed to encode jwt");

        let cookie = Cookie::build((AUTH_COOKIE_NAME, jwt))
            .path("/")
            .http_only(true)
            .max_age(Duration::seconds(AUTH_EXP as i64))
            .build();

        let mut response = if let Some(next) = ctx.query.get("next") {
            make_see_other(next)
        }
        else {
            make_response(StatusCode::OK, "authorized")
        };

        let val = HeaderValue::from_str(&cookie.to_string()).expect("unable to build header value");
        response.headers_mut().insert("Set-Cookie", val);

        return response;
    }

    make_not_found()
}

#[callback]
pub fn logout(ctx: ApiContext) -> BoxFuture<'static, Response> {
    let mut response = if let Some(next) = ctx.query.get("next") {
        make_see_other(next)
    }
    else {
        make_response(StatusCode::OK, "logged out")
    };

    let cookie = Cookie::build((AUTH_COOKIE_NAME, "_"))
        .max_age(Duration::seconds(0))
        .path("/")
        .http_only(true)
        .build();
    let val = HeaderValue::from_str(&cookie.to_string()).expect("unable to build header value");
    response.headers_mut().insert("Set-Cookie", val);

    response
}
