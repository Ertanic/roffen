use crate::{
    api::{posts::Post, resources::ResourceInfo},
    lang::LangManager,
    resources::{GetPostsRequest, ResourceManager},
};
use futures_util::{Stream, StreamExt};
use log::{error, trace};
use std::{collections::HashMap, sync::Arc};
use tokio::{runtime::Handle, sync::RwLock};
use upon::{Engine, Value};
use vfs::VfsResult;
use crate::templates::render::render_component;

pub fn register_functions(engine: &mut Engine, resources: Arc<RwLock<ResourceManager>>, lang_manager: LangManager) {
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

    engine.add_function("get_components", {
        let resources = Arc::clone(&resources);
        move || {
            tokio::task::block_in_place(|| {
                let runtime = Handle::current();
                let result = match runtime.block_on(async { resources.read().await.load_components().await }) {
                    Ok(comps) => runtime.block_on(async { comps.map(|comp| (comp.name.clone(), comp)).collect::<HashMap<_, _>>().await }),
                    Err(err) => {
                        error!("unable to load components because {err}");
                        return None;
                    }
                };
                upon::to_value(result).ok()
            })
        }
    });

    engine.add_function("get_component", |name: &str, components: &Value| {
        if let Value::Map(map) = components {
            map.get(name).cloned()
        }
        else {
            None
        }
    });

    engine.add_function("is_map", |val: &Value| matches!(val, Value::Map(_)));

    engine.add_function("flat_input", |val: &Value| {
        if let Value::Map(map) = val {
            if let Some((_, map)) = map.first_key_value() {
                Some(map.clone())
            }
            else {
                None
            }
        }
        else {
            None
        }
    });

    engine.add_function("len", |list: &[Value]| list.len() as i64);

    engine.add_function("date", |timestamp: i64, format: &str| {
        let timestamp = jiff::Timestamp::from_second(timestamp).ok()?;
        Some(timestamp.strftime(format).to_string())
    });

    engine.add_function("eq", |first: &Value, second: &Value| *first == *second);

    engine.add_function("and", |first: &Value, second: &Value| {
        if let (Value::Bool(first), Value::Bool(second)) = (first, second) {
            *first && *second
        }
        else {
            !matches!(first, Value::None) && !matches!(second, Value::None)
        }
    });

    engine.add_function("get_pages", {
        let resources = Arc::clone(&resources);
        move || {
            tokio::task::block_in_place(|| {
                let runtime = Handle::current();
                let result = match runtime.block_on(async { resources.read().await.get_pages().await }) {
                    Ok(pages) => runtime.block_on(async { pages.collect::<Vec<_>>().await }),
                    Err(err) => {
                        error!("unable to get pages list because {err}");
                        return None;
                    }
                };
                upon::to_value(result).ok()
            })
        }
    });

    engine.add_function("get_resources", {
        let resources = Arc::clone(&resources);
        move |path: &Value| {
            tokio::task::block_in_place(|| {
                let runtime = Handle::current();
                let path = if let Value::String(path) = path { path.as_str() } else { "/" };
                let result = match runtime.block_on(async { resources.read().await.get_resources_in_folder(path).await }) {
                    Ok(resources) => runtime.block_on(async { resources.collect::<Vec<_>>().await }),
                    Err(err) => {
                        error!("unable to get resources list because {err}");
                        return upon::to_value(Vec::<ResourceInfo>::new()).ok();
                    }
                };
                upon::to_value(result).ok()
            })
        }
    });

    engine.add_function(
        "default",
        |current: &Value, def: &Value| if matches!(current, Value::None) { def.clone() } else { current.clone() },
    );

    engine.add_function("split_path", |path: &str| {
        let path = path.trim_matches('/');

        if path.is_empty() {
            return upon::to_value(Vec::<(String, String)>::new()).ok();
        }

        let components = path.split('/').collect::<Vec<_>>();
        let mut result = Vec::with_capacity(components.len());
        for (i, comp) in components.iter().enumerate() {
            let mut link = String::new();
            let mut j = 0;
            while j < i {
                link.push('/');
                link.push_str(components[j]);
                j += 1;
            }
            link.push('/');
            link.push_str(comp);
            result.push((comp, link));
        }
        upon::to_value(result).ok()
    });

    engine.add_function("take", |vec: &[Value], count: usize| {
        let max = vec.len().min(count);
        Some(vec[..max].to_vec())
    });

    engine.add_function("lang", move |key: &str| {
        tokio::task::block_in_place(|| {
            let runtime = Handle::current();
            runtime.block_on(async { lang_manager.try_get_message(key).await })
        })
    });

    engine.add_function("render_component", render_component);
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
