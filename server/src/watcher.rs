use crate::{
    ResourceRefType,
    config::ArcConfig,
    consts::{CONFIG_FILENAME, LANG_FOLDER, LANG_META_FILE, PAGES_FOLDER, PUBLIC_FOLDER},
    lang::LangManager,
    resources::ResourceManager,
    routing::{MethodRouter, get},
    vfs::VfsPath,
};
use log::{debug, error, info, trace, warn};
use notify::{
    Event, EventKind, RecursiveMode, Watcher,
    event::{ModifyKind, RenameMode},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;

type ArcFabric = Arc<dyn EventFabric + Send + Sync + 'static>;

trait EventFabric {
    fn created(&self) -> ContentEventKind;
    fn modified(&self) -> ContentEventKind;
    fn removed(&self) -> ContentEventKind;
}

struct ConfigEventFabric;

impl EventFabric for ConfigEventFabric {
    fn created(&self) -> ContentEventKind {
        ContentEventKind::NewConfig
    }

    fn modified(&self) -> ContentEventKind {
        ContentEventKind::NewConfig
    }

    fn removed(&self) -> ContentEventKind {
        ContentEventKind::NewConfig
    }
}

struct PublicEventFabric;

impl EventFabric for PublicEventFabric {
    fn created(&self) -> ContentEventKind {
        ContentEventKind::NewPublic
    }

    fn modified(&self) -> ContentEventKind {
        ContentEventKind::NewPublic
    }

    fn removed(&self) -> ContentEventKind {
        ContentEventKind::RemovePublic
    }
}

struct PagesEventFabric;

impl EventFabric for PagesEventFabric {
    fn created(&self) -> ContentEventKind {
        ContentEventKind::NewPage
    }

    fn modified(&self) -> ContentEventKind {
        ContentEventKind::NewPage
    }

    fn removed(&self) -> ContentEventKind {
        ContentEventKind::RemovePage
    }
}

struct LangEventFabric;

impl EventFabric for LangEventFabric {
    fn created(&self) -> ContentEventKind {
        ContentEventKind::NewLang
    }

    fn modified(&self) -> ContentEventKind {
        ContentEventKind::NewLang
    }

    fn removed(&self) -> ContentEventKind {
        ContentEventKind::RemoveLang
    }
}

#[derive(Debug, PartialEq)]
pub enum ContentEventKind {
    NewConfig,
    NewPublic,
    NewPage,
    NewLang,
    RemovePublic,
    RemovePage,
    RemoveLang,
}

impl ContentEventKind {
    pub fn is_public(&self) -> bool {
        matches!(self, ContentEventKind::NewPublic) || matches!(self, ContentEventKind::RemovePublic)
    }
}

#[derive(Debug)]
pub enum ContentEvent {
    Change { kind: ContentEventKind, path: VfsPath },
    None,
}

impl ContentEvent {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn unwrap_path(&self) -> &VfsPath {
        match self {
            ContentEvent::Change { path, .. } => path,
            ContentEvent::None => panic!("unwrap_path() called on ContentEvent::None"),
        }
    }
}

#[derive(Clone)]
pub struct InitWatcherContext {
    pub router: Arc<RwLock<MethodRouter>>,
    pub resources: Arc<RwLock<ResourceManager>>,
    pub config: ArcConfig,
    pub lang_manager: LangManager,
}

pub type FilesListener = crossbeam_channel::Receiver<notify::Result<Event>>;

pub struct FilesWatcher {
    _watcher: Arc<dyn Watcher + Send + Sync>,
    listener: FilesListener,
    root: PathBuf,
    fabric: ArcFabric,
}

impl Iterator for &mut FilesWatcher {
    type Item = ContentEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.listener.recv().ok()?.ok()?;
        let event = notify_event_to_content_event(self.fabric.clone(), &self.root, event);

        if event.is_none() { None } else { Some(event) }
    }
}

fn notify_event_to_content_event(fabric: ArcFabric, root: &Path, e: Event) -> ContentEvent {
    let Some(path) = e.paths.into_iter().filter_map(|p| real_path_to_vfs(root, &p)).next()
    else {
        warn!("none path in filesystem event");
        return ContentEvent::None;
    };

    match e.kind {
        EventKind::Create(kind) => {
            trace!("file {path} created, kind: {kind:?}");
            ContentEvent::Change {
                kind: fabric.created(),
                path,
            }
        }
        EventKind::Remove(kind) => {
            trace!("file {path} removed, kind: {kind:?}");
            ContentEvent::Change {
                kind: fabric.removed(),
                path,
            }
        }
        EventKind::Modify(kind) => match kind {
            ModifyKind::Name(rename_mode) => {
                debug!("rename mode: {rename_mode:?}");
                match rename_mode {
                    RenameMode::From => ContentEvent::Change {
                        kind: fabric.removed(),
                        path,
                    },
                    RenameMode::To => ContentEvent::Change {
                        kind: fabric.created(),
                        path,
                    },
                    _ => ContentEvent::None,
                }
            }
            ModifyKind::Data(_) => {
                debug!("file {path} modified, kind: {kind:?}");
                ContentEvent::Change {
                    kind: fabric.modified(),
                    path,
                }
            }
            _ => ContentEvent::None,
        },
        _ => ContentEvent::None,
    }
}

fn real_path_to_vfs(root: &Path, path: &Path) -> Option<VfsPath> {
    path.strip_prefix(root).map(|p| VfsPath::new(p.to_string_lossy().replace('\\', "/"))).ok()
}

pub fn init_watcher(content: &Path, context: InitWatcherContext) {
    watch_folder(PagesEventFabric, &content.join(PAGES_FOLDER), context.clone());
    watch_folder(PublicEventFabric, &content.join(PUBLIC_FOLDER), context.clone());
    watch_folder(LangEventFabric, &content.join(LANG_FOLDER), context.clone());
    watch_folder(ConfigEventFabric, &content.join(CONFIG_FILENAME), context);
}

fn watch_folder<F>(fabric: F, path: &Path, context: InitWatcherContext)
where
    F: EventFabric + Send + Sync + 'static,
{
    let (tx, rx) = crossbeam_channel::unbounded();

    let mut watcher = match notify::recommended_watcher(tx) {
        Ok(w) => w,
        Err(err) => {
            error!(
                "failed to initialize files watcher in {} because {err}, changes to files will not be applied until the server is restarted",
                path.display()
            );
            return;
        }
    };

    match watcher.watch(path, RecursiveMode::Recursive) {
        Ok(_) => {
            info!("watcher initialized at {}", path.display());
            let watcher = FilesWatcher {
                root: path.to_owned(),
                _watcher: Arc::new(watcher),
                listener: rx,
                fabric: Arc::new(fabric),
            };

            tokio::spawn(async move {
                watch_content(watcher, context).await;
            });
        }
        Err(err) => {
            error!("failed to initialize watcher on {} because {err}", path.display());
        }
    }
}

async fn watch_content(mut watcher: FilesWatcher, context: InitWatcherContext) {
    loop {
        for event in &mut watcher {
            if let ContentEvent::Change { path, kind } = event {
                match kind {
                    ContentEventKind::NewPublic => {
                        let mut router = context.router.write().await;
                        let resource = ResourceRefType::File(VfsPath::new(PUBLIC_FOLDER).join(path.clone()));
                        if let Err(err) = router.try_add(get(&path), resource) {
                            error!("unable to update router with route {path} because {err}");
                        }
                    }
                    ContentEventKind::NewPage => {
                        let mut router = context.router.write().await;
                        context
                            .resources
                            .read()
                            .await
                            .load_page(&mut router, VfsPath::new(PAGES_FOLDER).join(path))
                            .await
                    }
                    ContentEventKind::RemovePublic | ContentEventKind::RemovePage => {
                        let mut router = context.router.write().await;
                        router.remove(get(&path))
                    }
                    ContentEventKind::NewLang | ContentEventKind::RemoveLang => {
                        let file = VfsPath::new(LANG_FOLDER).join(path);

                        if file.filename().starts_with(LANG_META_FILE) {
                            info!("language meta {file} has been changed");

                            let new_meta = match context.resources.read().await.load_lang_meta(&VfsPath::new(LANG_FOLDER)).await {
                                Ok(meta) => meta,
                                Err(err) => {
                                    error!("unable to load language meta {file} because {err}");
                                    continue;
                                }
                            };
                            context.lang_manager.replace_meta(new_meta).await;
                        }
                        else {
                            info!("language {file} has been changed");

                            let mut path = file;
                            while let Some(parent) = path.parent()
                                && parent.filename() != LANG_FOLDER
                            {
                                path = parent;
                            }

                            let bundle = match context.resources.read().await.load_lang(&path).await {
                                Ok(bundle) => bundle,
                                Err(err) => {
                                    error!("unable to load language {path} because {err}");
                                    continue;
                                }
                            };

                            context.lang_manager.replace_lang(path.filename(), bundle).await;
                        }
                    }
                    ContentEventKind::NewConfig => {
                        info!("config has been changed");
                        let new_config = match context.resources.read().await.try_load_config().await {
                            Ok(c) => c,
                            Err(err) => {
                                error!("unable to load config because {err}");
                                continue;
                            }
                        };
                        debug!("config loaded, try set new lang");
                        context.lang_manager.set_current(new_config.lang.current.clone()).await;
                        debug!("try set new default lang");
                        context.lang_manager.set_default(new_config.lang.default.clone()).await;
                        debug!("try set new config");
                        *context.config.write().await = new_config;
                        info!("new config loaded");
                    }
                }
            }
        }
    }
}
