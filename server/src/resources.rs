use crate::{
    PageLayout, ResourceRefType, Response,
    api::{
        ApiContext,
        components::{Component, ComponentMeta},
        posts::{Post, PostBody},
        resources::{ResourceInfo, ResourceType},
    },
    config::Config,
    consts::{
        COMPS_FOLDER, COMPS_JS_FILE, COMPS_META_FILE, CONFIG_FILENAME, INDEX_FILENAME, LANG_FOLDER, LANG_META_FILE, PAGES_FOLDER, POSTS_FOLDER,
        PUBLIC_FOLDER,
    },
    lang::{LangBundle, LangManager, LangMeta},
    routing::{MethodRouter, get},
    vfs::{PageDir, VfsPath, VirtualFS},
};
use dashmap::DashMap;
use fluent::{FluentResource, concurrent::FluentBundle};
use futures_util::{
    AsyncReadExt, AsyncWriteExt, Stream,
    future::BoxFuture,
    stream,
    stream::{BoxStream, StreamExt},
};
use log::{debug, error, info, trace, warn};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, env, path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use unic_langid::LanguageIdentifier;
use vfs::{VfsFileType, VfsResult, async_vfs::AsyncFileSystem, error::VfsErrorKind};

pub trait BoxedApiCallback {
    fn boxed(self) -> Box<dyn Fn(ApiContext) -> BoxFuture<'static, Response> + Send + Sync + 'static>;
}

impl<F> BoxedApiCallback for F
where
    F: Fn(ApiContext) -> BoxFuture<'static, Response> + Send + Sync + 'static,
{
    fn boxed(self) -> Box<dyn Fn(ApiContext) -> BoxFuture<'static, Response> + Send + Sync + 'static> {
        Box::new(self)
    }
}

pub fn api(callback: impl BoxedApiCallback) -> ResourceRefType {
    ResourceRefType::Api(callback.boxed())
}

#[derive(Serialize)]
pub struct PageInfo {
    pub link: String,
    pub path: VfsPath,
    pub tags: Vec<String>,
}

pub enum GetPostsRequest {
    Full,
    Chunk { count: usize, offset: usize },
}

#[derive(Deserialize, Clone)]
pub struct TlsContext {
    pub cert: PathBuf,
    pub key: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct SecurityContext {
    pub tls: TlsContext,
}

pub struct ResourceManager {
    vfs: VirtualFS,
}

impl ResourceManager {
    pub fn new(vfs: VirtualFS) -> Self {
        Self { vfs }
    }

    pub async fn load_config(&self) -> Config {
        self.try_load_config().await.expect("load config failed")
    }

    pub async fn try_load_config(&self) -> VfsResult<Config> {
        let filepath = VfsPath::new(CONFIG_FILENAME);
        let mut buf = String::new();

        self.vfs.open_file(&filepath).await?.read_to_string(&mut buf).await?;

        let config = toml::from_str(&buf).map_err(|e| VfsErrorKind::Other(e.to_string()))?;

        info!("config {filepath} has been loaded");

        Ok(config)
    }

    pub async fn load_pages(&self, router: &mut MethodRouter) {
        let mut pages = vec![PageDir::new(VfsPath::new(PAGES_FOLDER))];

        while let Some(page) = pages.pop() {
            self._load_page(router, &mut pages, page).await
        }
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

        match router.try_add(get(&normalized), resource) {
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

    pub async fn load_public(&self, router: &mut MethodRouter) {
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

                    match router.try_add(get(&normalized), ResourceRefType::File(VfsPath::new(entry))) {
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

    pub async fn get_pages(&self) -> VfsResult<BoxStream<'static, PageInfo>> {
        let vfs = Arc::clone(&self.vfs);

        let state = (
            VecDeque::from([VfsPath::new(PAGES_FOLDER)]),
            None::<(VfsPath, Box<dyn Unpin + Stream<Item = String> + Send>)>,
        );

        let stream = stream::unfold(state, move |(mut folders, mut current)| {
            let vfs = Arc::clone(&vfs);

            async move {
                loop {
                    if current.is_none() {
                        let folder = folders.pop_front()?;

                        match vfs.read_dir(&folder).await {
                            Ok(reader) => {
                                current = Some((folder, reader));
                            }
                            Err(err) => {
                                error!("unable to read folder {folder} because {err}");
                                continue;
                            }
                        }
                    }

                    let (folder, reader) = current.as_mut().unwrap();

                    match reader.next().await {
                        Some(entry) => {
                            let path = folder.join(entry);
                            let meta = match vfs.metadata(&path).await {
                                Ok(meta) => meta,
                                Err(err) => {
                                    error!("unable to read metadata for {path} because {err}");
                                    continue;
                                }
                            };

                            if meta.file_type == VfsFileType::Directory {
                                trace!("found folder {path}");
                                folders.push_back(path);
                                continue;
                            }

                            if path.filename() != INDEX_FILENAME {
                                continue;
                            }

                            let normalized = path.strip_prefix(VfsPath::new(PAGES_FOLDER)).and_then(|p| p.parent());
                            let Some(normalized) = normalized
                            else {
                                continue;
                            };

                            let mut tags = Vec::with_capacity(1);

                            if normalized.starts_with(&VfsPath::new("admin")) {
                                tags.push("admin-panel-pages-tag-system".to_owned());
                            }
                            else {
                                tags.push("admin-panel-pages-tag-user".to_owned())
                            }

                            let link = normalized.as_string();

                            return Some((PageInfo { link, path, tags }, (folders, current)));
                        }

                        None => {
                            trace!("no more files in folder {folder}");
                            current = None;
                        }
                    }
                }
            }
        })
        .boxed();

        Ok(stream)
    }

    async fn ensure_lang_folder(&self) -> VfsResult<VfsPath> {
        let folder = VfsPath::new(LANG_FOLDER);

        if let Ok(exists) = self.vfs.exists(&folder).await
            && !exists
        {
            self.vfs.create_dir(LANG_FOLDER).await?;
        }

        Ok(folder)
    }

    pub async fn load_lang_meta(&self, lang_folder: &VfsPath) -> VfsResult<LangMeta> {
        let meta = lang_folder.join(LANG_META_FILE);
        let mut meta_content = String::new();
        self.vfs.open_file(&meta).await?.read_to_string(&mut meta_content).await?;
        toml::from_str(&meta_content).map_err(|err| VfsErrorKind::Other(err.to_string()).into())
    }

    pub async fn load_lang(&self, lang_folder: &VfsPath) -> VfsResult<LangBundle> {
        let lang_id = match lang_folder.filename().parse::<LanguageIdentifier>() {
            Ok(lang_id) => lang_id,
            Err(err) => {
                error!("unable to parse language identifier {} because {err}", lang_folder.filename());
                return Err(VfsErrorKind::Other(err.to_string()).into());
            }
        };

        let stack = Arc::new(Mutex::new(vec![lang_folder.clone()]));
        let bundle = Arc::new(RwLock::new(FluentBundle::new_concurrent(vec![lang_id])));

        while let Some(path) = stack.lock().await.pop() {
            match self.vfs.read_dir(&path).await {
                Ok(reader) => reader.for_each(|e| {
                    let bundle = bundle.clone();
                    let stack = stack.clone();
                    let path = path.clone();

                    async move {
                        let entry = path.join(e);

                        if let Ok(meta) = self.vfs.metadata(&entry).await
                            && matches!(meta.file_type, VfsFileType::Directory)
                        {
                            stack.lock().await.push(entry);
                            return;
                        }

                        let mut content = String::new();
                        let mut file = match self.vfs.open_file(&entry).await {
                            Ok(file) => file,
                            Err(err) => {
                                error!("unable to open file {entry} because {err}");
                                return;
                            }
                        };

                        match file.read_to_string(&mut content).await {
                            Ok(_) => {}
                            Err(err) => {
                                error!("unable to read file {entry} because {err}");
                                return;
                            }
                        }

                        let resource = match FluentResource::try_new(content) {
                            Ok(resource) => resource,
                            Err(err) => {
                                error!("unable to parse resource {entry} because {:?}", err.1);
                                return;
                            }
                        };

                        match bundle.write().await.add_resource(resource) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("unable to add resource {entry} because {err:?}");
                            }
                        };
                    }
                }),
                Err(err) => {
                    error!("unable to read directory {path} because {err}");
                    return Err(err);
                }
            }
            .await;
        }

        Ok(bundle)
    }

    pub async fn load_langs(&self, default_lang: String, current_lang: String) -> VfsResult<LangManager> {
        let folder = self.ensure_lang_folder().await?;
        let meta = self.load_lang_meta(&folder).await?;

        let map = self
            .vfs
            .read_dir(&folder)
            .await?
            .fold(DashMap::new(), |map, entry| async {
                if entry == LANG_META_FILE {
                    return map;
                }

                let bundle = match self.load_lang(&folder.join(VfsPath::new(&entry))).await {
                    Ok(bundle) => bundle,
                    Err(err) => {
                        error!("unable to load language {entry} because {err}");
                        return map;
                    }
                };

                map.insert(entry, bundle);

                map
            })
            .await;

        Ok(LangManager::new(current_lang, default_lang, map, meta.lang))
    }

    pub async fn get_resources_in_folder(&self, folder: &str) -> VfsResult<BoxStream<'static, ResourceInfo>> {
        let folder = VfsPath::new(PUBLIC_FOLDER).join(folder);
        trace!("getting resources in {folder} folder");

        let exists = self.vfs.exists(&folder).await?;
        if !exists {
            return Err(VfsErrorKind::FileNotFound.into());
        }

        let meta = self.vfs.metadata(&folder).await?;
        if meta.file_type != VfsFileType::Directory {
            return Err(VfsErrorKind::InvalidPath.into());
        }

        let vfs = Arc::clone(&self.vfs);
        let stream = vfs
            .read_dir(&folder)
            .await?
            .map(move |f| folder.join(f))
            .filter_map(move |f| {
                let vfs = Arc::clone(&vfs);
                async move {
                    let meta = vfs.metadata(&f).await.ok()?;
                    let link = f.strip_prefix(VfsPath::new(PUBLIC_FOLDER))?.as_string();
                    Some(ResourceInfo {
                        name: f.filename(),
                        path: f.as_string(),
                        link,
                        resource_type: if matches!(meta.file_type, VfsFileType::Directory) {
                            ResourceType::Directory
                        }
                        else {
                            ResourceType::File
                        },
                    })
                }
            })
            .boxed();

        Ok(stream)
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
