use crate::{
    Response,
    api::ApiContext,
    resources::GetPostsRequest,
    utils,
    utils::{make_bad_request, make_internal_error, make_json_response, make_no_content, make_not_found, make_unauthorized},
};
use futures_util::{StreamExt, future::BoxFuture, stream, stream::BoxStream};
use http_body_util::{BodyExt, StreamBody};
use hyper::{
    StatusCode,
    body::{Bytes, Frame},
};
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use small_uid::SmallUid;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use vfs::VfsError;

#[derive(Debug, Serialize, Deserialize)]
struct PostComponent {
    name: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    data: HashMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    children: Vec<PostComponent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    #[serde(flatten)]
    pub content: PostBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostBody {
    draft: bool,
    author: String,
    created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<u64>,
    title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    content: Vec<PostComponent>,
}

#[derive(Deserialize)]
struct CreatePostRequest {
    title: String,
    status: PostStatus,
    #[serde(default)]
    content: Vec<PostComponent>,
}

#[derive(Serialize)]
struct CreatePostResponse {
    post_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum PostStatus {
    Public,
    Draft,
}

impl PostStatus {
    fn is_draft(&self) -> bool {
        matches!(self, PostStatus::Draft)
    }
}

#[derive(Deserialize)]
struct UpdatePostRequestUpdates {
    new_status: Option<PostStatus>,
    new_title: Option<String>,
    new_content: Option<Vec<PostComponent>>,
}

#[derive(Deserialize)]
struct UpdatePostRequest {
    post_id: String,
    #[serde(flatten)]
    updates: UpdatePostRequestUpdates,
}

pub fn create_post(ctx: ApiContext) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        if ctx.jwt.is_none() {
            return make_unauthorized();
        }

        if ctx.request.headers().get("Content-Type") != Some(&"application/json".parse().unwrap()) {
            return make_bad_request();
        }

        let request = match ctx.request.into_body().collect().await {
            Ok(request) => request.to_bytes(),
            Err(err) => {
                error!("failed to collect request body because {err}");
                return make_bad_request();
            }
        };

        let request = String::from_utf8_lossy(request.as_ref());
        let request: CreatePostRequest = match serde_json::from_str(request.as_ref()) {
            Ok(request) => request,
            Err(err) => {
                error!("failed to parse request body because {err}");
                return make_bad_request();
            }
        };

        let id = SmallUid::new().to_string();
        let body = PostBody {
            draft: request.status.is_draft(),
            author: ctx.jwt.unwrap().username,
            created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            updated_at: None,
            title: request.title,
            content: request.content,
        };
        let post = Post { id, content: body };

        if ctx.resources.read().await.save_post(&post).await.is_err() {
            return make_internal_error();
        }

        let response_body = serde_json::to_string(&CreatePostResponse { post_id: post.id }).unwrap();
        utils::make_response(StatusCode::CREATED, &response_body)
    })
}

pub fn delete_post(ctx: ApiContext) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        if ctx.jwt.is_none() {
            return make_unauthorized();
        }

        let Some(post_id) = ctx.query.get("id")
        else {
            error!("no post id in query");
            return make_bad_request();
        };

        match ctx.resources.read().await.delete_post(post_id).await {
            Ok(_) => {
                info!("post {post_id} has been deleted by {}", ctx.jwt.unwrap().username);
            }
            Err(err) => {
                error!("unable to delete post {post_id} because {err}");
                return make_internal_error();
            }
        }

        make_no_content()
    })
}

pub fn update_post(ctx: ApiContext) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        if ctx.jwt.is_none() {
            return make_unauthorized();
        }

        let request = match ctx.request.into_body().collect().await {
            Ok(request) => request.to_bytes(),
            Err(err) => {
                error!("failed to collect request body because {err}");
                return make_internal_error();
            }
        };

        let request: UpdatePostRequest = match serde_json::from_slice(&request) {
            Ok(request) => request,
            Err(err) => {
                error!("failed to parse request body because {err}");
                return make_bad_request();
            }
        };

        let mut post = match ctx.resources.read().await.get_post(&request.post_id).await {
            Ok(post) => post,
            Err(err) => {
                error!("post file not found because {err}");
                return make_internal_error();
            }
        };

        let mut edited = false;
        if let Some(title) = request.updates.new_title {
            edited = true;
            post.content.title = title;
        }

        if let Some(status) = request.updates.new_status {
            edited = true;
            post.content.draft = status.is_draft();
        }

        if let Some(content) = request.updates.new_content {
            edited = true;
            post.content.content = content;
        }

        if edited {
            post.content
                .updated_at
                .replace(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

            if let Err(err) = ctx.resources.read().await.save_post(&post).await {
                error!("unable to save {} post file because {err}", post.id);
                return make_internal_error();
            }
        }

        make_no_content()
    })
}

pub fn get_posts(ctx: ApiContext) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        let resources = ctx.resources.read().await;

        if let Some(post_id) = ctx.query.get("id") {
            let Ok(post) = ctx.resources.read().await.get_post(post_id).await
            else {
                trace!("post not found");
                return make_not_found();
            };

            let body = serde_json::to_string(&post).unwrap();
            make_json_response(&body)
        }
        else if let (Some(count), Some(offset)) = (ctx.query.get("count"), ctx.query.get("offset")) {
            let (Ok(count), Ok(offset)) = (count.parse(), offset.parse())
            else {
                return make_bad_request();
            };

            trace!("fetched {count} posts with {offset} offset");
            make_response(resources.get_posts(GetPostsRequest::Chunk { count, offset }).await)
        }
        else {
            trace!("fetched full posts list");
            make_response(resources.get_posts(GetPostsRequest::Full).await)
        }
    })
}

fn make_response(stream: Result<BoxStream<'static, Post>, VfsError>) -> Response {
    let posts = match stream {
        Ok(posts) => build_json(posts),
        Err(err) => {
            error!("failed to read posts folder because {err}");
            return make_internal_error();
        }
    };

    let mut response = Response::new(StreamBody::new(Box::pin(posts)));
    response.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
    response
}

fn build_json(stream: BoxStream<'static, Post>) -> crate::BoxStream {
    stream::once(async { str_to_frame("[") })
        .chain(stream.enumerate().flat_map(|(i, post)| {
            if i == 0 {
                stream::once(async move { post_to_frame(post) }).boxed()
            }
            else {
                stream::iter([str_to_frame(","), post_to_frame(post)]).boxed()
            }
        }))
        .chain(stream::once(async { str_to_frame("]") }))
        .boxed()
}

fn post_to_frame(post: Post) -> Result<Frame<Bytes>, std::io::Error> {
    let body = serde_json::to_string(&post)?;
    let bytes = Bytes::from(body);
    let frame = Frame::data(bytes);
    Ok(frame)
}

fn str_to_frame(str: &'static str) -> Result<Frame<Bytes>, std::io::Error> {
    Ok(Frame::data(Bytes::from_static(str.as_ref())))
}
