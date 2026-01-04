use crate::{
    api::posts::Post,
    resources::{GetPostsRequest, ResourceManager},
};
use futures_util::{Stream, StreamExt};
use log::{error, trace};
use std::sync::Arc;
use tokio::{runtime::Handle, sync::RwLock};
use upon::{Engine, Value};
use vfs::VfsResult;

pub fn register_functions(engine: &mut Engine, resources: Arc<RwLock<ResourceManager>>) {
    trace!("registering templates functions...");

    engine.add_function("all_posts", {
        let resources = Arc::clone(&resources);
        move || {
            tokio::task::block_in_place(|| {
                let runtime = Handle::current();
                let result = runtime.block_on(async { resources.read().await.get_posts(GetPostsRequest::Full).await });
                posts_to_upon_values(runtime, result)
            })
        }
    });

    engine.add_function("posts", {
        let resources = Arc::clone(&resources);
        move |count: usize, offset: usize| {
            tokio::task::block_in_place(|| {
                let runtime = Handle::current();
                let result = runtime.block_on(async { resources.read().await.get_posts(GetPostsRequest::Chunk { count, offset }).await });
                posts_to_upon_values(runtime, result)
            })
        }
    });

    engine.add_function("get_post_by_id", {
        let resources = Arc::clone(&resources);
        move |id: &str| {
            tokio::task::block_in_place(|| {
                let runtime = Handle::current();
                let result = match runtime.block_on(async { resources.read().await.get_post(id).await }) {
                    Ok(post) => post,
                    Err(err) => {
                        error!("unable to get post {id} because {err}");
                        return None;
                    }
                };
                upon::to_value(result).ok()
            })
        }
    });

    engine.add_function("len", |list: &[Value]| list.len() as i64);

    engine.add_function("date", |timestamp: i64, format: &str| {
        let timestamp = jiff::Timestamp::from_second(timestamp).ok()?;
        Some(timestamp.strftime(format).to_string())
    });

    engine.add_function("eq", |first: &Value, second: &Value| *first == *second);
}

fn posts_to_upon_values(runtime: Handle, stream_result: VfsResult<impl Stream<Item = Post>>) -> Option<Vec<Value>> {
    let posts = match stream_result {
        Ok(posts) => posts.filter_map(async |p| upon::to_value(p).ok()),
        Err(err) => {
            error!("unable to get posts list because {err}");
            return None;
        }
    };

    Some(runtime.block_on(posts.collect::<Vec<_>>()))
}
