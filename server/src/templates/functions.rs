use crate::{
    api::posts::Post,
    resources::{GetPostsRequest, ResourceManager},
};
use futures_util::{Stream, StreamExt};
use log::{debug, error};
use std::sync::Arc;
use tokio::{runtime::Handle, sync::RwLock};
use upon::{Engine, Value};
use vfs::VfsResult;

pub fn register_functions(engine: &mut Engine, resources: Arc<RwLock<ResourceManager>>) {
    debug!("registering templates functions...");

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
