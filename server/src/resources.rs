use crate::{
    PageLayout, ResourceRefType,
    api::auth::AuthContext,
    consts::{AUTH_FILENAME, INDEX_FILENAME, PAGES_FOLDER, PUBLIC_FOLDER},
    routing::{MethodRouter, get},
    vfs::{PageDir, VfsPath, VirtualFS},
};
use futures_util::AsyncReadExt;
use log::{debug, warn};
use std::{env, path::PathBuf};
use tokio_stream::StreamExt;
use vfs::{VfsFileType, async_vfs::AsyncFileSystem};

pub struct ResourceManager {
    vfs: VirtualFS,
}

impl ResourceManager {
    pub fn new(vfs: VirtualFS) -> Self {
        Self { vfs }
    }

    pub async fn load_auth(&self) -> AuthContext {
        let mut buf = String::new();
        let filepath = VfsPath::new(AUTH_FILENAME);

        self.vfs
            .open_file(&filepath)
            .await
            .expect("unable to open auth context")
            .read_to_string(&mut buf)
            .await
            .expect("unable to read auth config file");

        let auth = toml::from_str(&buf).expect("unable to parse auth config file");

        debug!("auth context loaded");

        auth
    }

    pub async fn load_pages(&self, mut router: MethodRouter) -> MethodRouter {
        let mut pages = vec![PageDir::new(VfsPath::new(PAGES_FOLDER))];

        while let Some(page) = pages.pop() {
            let mut children = vec![];
            let mut index_file = None;
            let mut layouts = vec![];

            let mut page_reader = match self.vfs.read_dir(&page.path).await {
                Ok(r) => r,
                Err(err) => {
                    warn!("unable to read folder {} because {err}, skipping", page.path);
                    continue;
                }
            };

            while let Some(entry) = page_reader.next().await {
                let entry = VfsPath::from([page.path.clone(), entry.into()]);

                let filetype = match self.vfs.metadata(&entry).await {
                    Ok(t) => t,
                    Err(err) => {
                        warn!("unable to get file type {entry} because {err}, skipping");
                        continue;
                    }
                };

                if filetype.file_type == VfsFileType::Directory {
                    children.push(PageDir::new(entry));
                }
                else {
                    let filename = entry.split('/').next_back().unwrap().to_owned();
                    if filename == INDEX_FILENAME {
                        index_file = Some(entry);
                    }
                    else {
                        let layout = PageLayout::new(entry);
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

            layouts.extend(page.inherited_layouts);

            let resource = ResourceRefType::Page {
                index: index_file.unwrap(),
                layouts,
            };

            let normalized = normalize_route(&page.path.strip_prefix(VfsPath::new(PAGES_FOLDER)).unwrap());

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

    pub async fn load_public(&self, mut router: MethodRouter) -> MethodRouter {
        let mut dirs = vec![VfsPath::new(PUBLIC_FOLDER)];

        while let Some(dir) = dirs.pop() {
            let mut folder_reader = match self.vfs.read_dir(&dir).await {
                Ok(reader) => reader,
                Err(err) => {
                    warn!("unable to read folder {dir} because {err}, skipping");
                    continue;
                }
            };

            while let Some(entry) = folder_reader.next().await {
                let entry = VfsPath::from([dir.clone(), entry.into()]);

                let filetype = match self.vfs.metadata(&entry).await {
                    Ok(t) => t,
                    Err(err) => {
                        warn!("unable to get file type of {entry} because {err}, skipping");
                        continue;
                    }
                };

                if filetype.file_type == VfsFileType::Directory {
                    dirs.push(entry);
                }
                else {
                    let normalized = normalize_route(&entry.strip_prefix(VfsPath::new(PUBLIC_FOLDER)).unwrap());

                    match router.add(get(&normalized), ResourceRefType::File(VfsPath::new(entry))) {
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
}

fn normalize_route(route: &str) -> String {
    if route.starts_with("/") {
        route.to_string()
    }
    else {
        "/".to_owned() + route
    }
}

pub fn get_root() -> PathBuf {
    env::current_exe()
        .expect("unable to get app path")
        .parent()
        .expect("no app parent dir")
        .to_path_buf()
}
