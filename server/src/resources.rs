use crate::{
    PageDir, PageLayout, ResourceRefType,
    api::auth::AuthContext,
    consts::INDEX_FILENAME,
    routing::{MethodRouter, get},
};
use log::{debug, warn};
use std::{
    env,
    path::{Path, PathBuf},
};

pub async fn load_auth(auth_config_path: &Path) -> AuthContext {
    let auth_context_content = tokio::fs::read_to_string(&auth_config_path)
        .await
        .expect("unable to read auth config file");
    toml::from_str(&auth_context_content).expect("unable to parse auth config file")
}

pub async fn load_pages(pages_root: &Path, mut router: MethodRouter) -> MethodRouter {
    let mut pages = vec![PageDir::new(pages_root.to_path_buf())];

    while let Some(page) = pages.pop() {
        let mut children = vec![];
        let mut index_file = None;
        let mut layouts = vec![];

        let mut page_reader = match tokio::fs::read_dir(&page.path).await {
            Ok(r) => r,
            Err(err) => {
                warn!("unable to read folder {:?} because {err}, skipping", page.path);
                continue;
            }
        };

        while let Ok(Some(entry)) = page_reader.next_entry().await {
            let filetype = match entry.file_type().await {
                Ok(t) => t,
                Err(err) => {
                    warn!("unable to get file type {:?} because {err}, skipping", entry.path());
                    continue;
                }
            };

            if filetype.is_dir() {
                children.push(PageDir::new(entry.path()));
            }
            else {
                let filename = entry.file_name().to_string_lossy().to_string();
                if filename == INDEX_FILENAME {
                    index_file = Some(entry.path());
                }
                else {
                    let layout = PageLayout::new(entry.path());
                    layouts.push(layout);
                }
            }
        }

        while let Some(mut child) = children.pop() {
            child.inherited_layouts.extend(layouts.iter().cloned());
            pages.push(child);
        }

        if index_file.is_none() {
            warn!("{INDEX_FILENAME} not found in page folder {:?}", page.path);
            continue;
        }

        let name = page
            .path
            .strip_prefix(pages_root)
            .expect("failed to strip prefix")
            .to_string_lossy()
            .to_string();
        let normalized = normalize_route(&name);

        layouts.extend(page.inherited_layouts);

        let resource = ResourceRefType::Page {
            index: index_file.unwrap(),
            layouts,
        };

        match router.add(get(&normalized), resource) {
            Ok(_) => {
                debug!("page route {normalized} has been registered")
            }
            Err(err) => {
                warn!("page route {normalized} is not registered because {err}");
            }
        }
    }

    router
}

pub async fn load_public(public_root: &Path, mut router: MethodRouter) -> MethodRouter {
    let mut dirs = vec![public_root.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        let mut folder_reader = match tokio::fs::read_dir(&dir).await {
            Ok(reader) => reader,
            Err(err) => {
                warn!("unable to read folder {dir:?} because {err}, skipping");
                continue;
            }
        };

        while let Ok(Some(entry)) = folder_reader.next_entry().await {
            let filetype = match entry.file_type().await {
                Ok(t) => t,
                Err(err) => {
                    warn!("unable to get file type of {:?} because {err}, skipping", entry.path());
                    continue;
                }
            };

            if filetype.is_dir() {
                dirs.push(entry.path());
            }
            else {
                let name = entry
                    .path()
                    .strip_prefix(public_root)
                    .expect("failed to strip prefix")
                    .to_string_lossy()
                    .to_string();
                let normalized = normalize_route(&name);

                match router.add(get(&normalized), ResourceRefType::File(entry.path())) {
                    Ok(_) => {
                        debug!("route \"{normalized}\" has been registered");
                    }
                    Err(err) => {
                        warn!("route \"{normalized}\" is not registered because {err}");
                    }
                }
            }
        }
    }

    router
}

fn normalize_route(route: &str) -> String {
    if route.starts_with("/") {
        route.to_string()
    }
    else {
        "/".to_owned() + route
    }
    .replace('\\', "/")
}

pub fn get_root() -> PathBuf {
    let app_path = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| env::current_exe())
        .expect("unable to get app path");
    app_path.parent().expect("no app parent dir").to_path_buf()
}
