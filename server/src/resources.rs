use crate::{
    PageLayout, ResourceRefType,
    api::{
        auth::AuthContext,
        components::{Component, ComponentMeta},
        posts::{Post, PostBody},
    },
    consts::{AUTH_FILENAME, COMPS_FOLDER, COMPS_JS_FILE, COMPS_META_FILE, INDEX_FILENAME, PAGES_FOLDER, POSTS_FOLDER, PUBLIC_FOLDER},
    routing::{MethodRouter, get},
    vfs::{PageDir, VfsPath, VirtualFS},
};
use futures_util::{
    AsyncReadExt, AsyncWriteExt, stream,
    stream::{BoxStream, StreamExt},
};
use log::{debug, error, warn};
use ron::ser::PrettyConfig;
use std::{env, path::PathBuf, sync::Arc};
use vfs::{VfsFileType, VfsResult, async_vfs::AsyncFileSystem, error::VfsErrorKind};

pub enum GetPostsRequest {
    Full,
    Chunk { count: usize, offset: usize },
}

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
            self._load_page(&mut router, &mut pages, page).await
        }

        router
    }

    async fn _load_page(&self, router: &mut MethodRouter, pages: &mut Vec<PageDir>, page: PageDir) {
        let mut children = vec![];
        let mut index_file = None;
        let mut layouts = vec![];

        let mut page_reader = match self.vfs.read_dir(&page.path).await {
            Ok(r) => r,
            Err(err) => {
                warn!("unable to read folder {} because {err}, skipping", page.path);
                return;
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
            child.inherited_layouts.extend(page.inherited_layouts.iter().cloned());
            child.inherited_layouts.extend(layouts.iter().cloned());
            pages.push(child);
        }

        if index_file.is_none() {
            warn!("{INDEX_FILENAME} not found in page folder {:?}", page.path);
            return;
        }

        layouts.extend(page.inherited_layouts);

        let resource = ResourceRefType::Page {
            index: index_file.unwrap(),
            layouts,
        };

        let normalized = normalize_route(&page.path.strip_prefix(VfsPath::new(PAGES_FOLDER)).unwrap());

        match router.add(get(&normalized), resource) {
            Ok(_) => {
                debug!("page route {normalized} has been registered");
            }
            Err(err) => {
                warn!("page route {normalized} is not registered because {err}");
            }
        }
    }

    pub async fn load_page(&self, router: &mut MethodRouter, path: VfsPath) {
        if let Some(parent) = path.parent() {
            let mut pages = vec![PageDir::new(parent)];

            while let Some(page) = pages.pop() {
                self._load_page(router, &mut pages, page.clone()).await;
                debug!("page {page:?} has been registered");
            }
        }
        else {
            warn!("unable to load page {path}");
        }
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

    async fn ensure_posts_folder(&self) -> VfsResult<VfsPath> {
        let folder = VfsPath::new(POSTS_FOLDER);

        if let Ok(exists) = self.vfs.exists(&folder).await
            && !exists
        {
            self.vfs.create_dir(&folder).await?;
        }

        Ok(folder)
    }

    pub async fn save_post(&self, post: &Post) -> VfsResult<()> {
        let posts_folder = self.ensure_posts_folder().await?;

        let filename = get_post_filename(&post.id);
        let path = posts_folder.join(filename);

        let content = ron::ser::to_string_pretty(&post.content, PrettyConfig::new().struct_names(true)).unwrap();

        // it is only possible to add to the contents of the file, but not to overwrite it
        if let Ok(exists) = self.vfs.exists(&path).await
            && exists
        {
            debug!("post file already exists, removing...");
            self.vfs.remove_file(&path).await?;
        }

        let mut file = self.vfs.create_file(&path).await?;
        file.write_all(content.as_bytes()).await?;

        Ok(())
    }

    pub async fn delete_post(&self, post_id: &str) -> VfsResult<()> {
        let posts_folder = self.ensure_posts_folder().await?;

        let filename = get_post_filename(post_id);
        let path = posts_folder.join(filename);

        if let Ok(exists) = self.vfs.exists(&path).await
            && exists
        {
            self.vfs.remove_file(&path).await?;
        }

        Ok(())
    }

    pub async fn get_posts(&self, request: GetPostsRequest) -> VfsResult<BoxStream<'static, Post>> {
        let posts_folder = self.ensure_posts_folder().await?;
        let vfs = VirtualFS::clone(&self.vfs);
        let mut folder_reader = vfs.read_dir(&posts_folder).await?.collect::<Vec<String>>().await;

        folder_reader.sort();

        let folder_reader = stream::iter(folder_reader.into_iter())
            .map(move |entry| posts_folder.join(entry))
            .filter_map(move |entry| {
                let vfs = Arc::clone(&vfs);
                async move {
                    let post = match read_post(vfs, &entry).await {
                        Ok(post) => post,
                        Err(err) => {
                            error!("unable to read post file {entry} because {err}");
                            return None;
                        }
                    };
                    Some(post)
                }
            });

        match request {
            GetPostsRequest::Full => Ok(Box::pin(folder_reader)),
            GetPostsRequest::Chunk { count, offset } => Ok(Box::pin(folder_reader.skip(offset * count).take(count))),
        }
    }

    pub async fn get_post(&self, post_id: &str) -> VfsResult<Post> {
        let posts_folder = self.ensure_posts_folder().await?;
        let filename = get_post_filename(post_id);
        let filepath = posts_folder.join(filename);

        read_post(self.vfs.clone(), &filepath).await
    }

    async fn ensure_components_folder(&self) -> VfsResult<VfsPath> {
        let folder = VfsPath::new(COMPS_FOLDER);

        if let Ok(exists) = self.vfs.exists(&folder).await
            && !exists
        {
            self.vfs.create_dir(COMPS_FOLDER).await?;
        }

        Ok(folder)
    }

    pub async fn load_components(&self) -> VfsResult<BoxStream<'static, Component>> {
        let comps_folder = self.ensure_components_folder().await?;
        let exists_vfs = Arc::clone(&self.vfs);
        let read_vfs = Arc::clone(&self.vfs);

        let components = self
            .vfs
            .read_dir(&comps_folder)
            .await?
            .map(move |f| comps_folder.join(f))
            .filter_map(move |f| {
                let vfs = Arc::clone(&exists_vfs);
                async move {
                    let meta_file = f.join(COMPS_META_FILE);
                    let js_file = f.join(COMPS_JS_FILE);

                    if let Ok(meta_exists) = vfs.exists(&meta_file).await
                        && meta_exists
                    {
                        if let Ok(js_exists) = vfs.exists(&meta_file).await
                            && js_exists
                        {
                            Some((meta_file, js_file))
                        }
                        else {
                            warn!("no {COMPS_JS_FILE} in component folder {f}");
                            None
                        }
                    }
                    else {
                        warn!("no meta in component folder {f}");
                        None
                    }
                }
            })
            .filter_map(move |(meta_path, js_path)| {
                let vfs = Arc::clone(&read_vfs);
                async move {
                    match read_component_meta(vfs, &meta_path).await {
                        Ok(meta) => Some(Component { js: js_path, meta }),
                        Err(err) => {
                            warn!("unable to read component metadata because {err}");
                            None
                        }
                    }
                }
            });

        Ok(Box::pin(components))
    }

    pub async fn load_component_js(&self, component: &Component) -> VfsResult<String> {
        let mut buf = String::new();
        let mut file = self.vfs.open_file(&component.js).await?;

        file.read_to_string(&mut buf).await?;

        Ok(buf)
    }
}

async fn read_component_meta(vfs: VirtualFS, meta: &str) -> VfsResult<ComponentMeta> {
    let mut buf = String::new();
    let mut file = vfs.open_file(meta).await?;

    file.read_to_string(&mut buf).await?;

    let meta = match ron::from_str(&buf) {
        Ok(meta) => meta,
        Err(err) => return Err(VfsErrorKind::Other(format!("{err}")).into()),
    };

    Ok(meta)
}

async fn read_post(vfs: VirtualFS, filename: &str) -> VfsResult<Post> {
    let mut content = String::new();
    vfs.open_file(filename).await?.read_to_string(&mut content).await?;

    let body: PostBody = match ron::from_str(&content) {
        Ok(body) => body,
        Err(err) => {
            error!("unable to read post body {filename} because {err}");
            return Err(VfsErrorKind::Other(format!("unable to read post body {filename} because {err}")).into());
        }
    };

    let id = filename.split('/').next_back().unwrap().split('.').next().unwrap().to_owned();

    Ok(Post { id, content: body })
}

fn get_post_filename(id: &str) -> String {
    format!("{}.ron", id)
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
    #[cfg(not(debug_assertions))]
    let path = env::current_exe()
        .expect("unable to get app path")
        .parent()
        .expect("no app parent dir")
        .to_path_buf();

    #[cfg(debug_assertions)]
    let path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("unable to get manifest dir"))
        .parent()
        .expect("no parent dir")
        .to_path_buf();

    path
}
