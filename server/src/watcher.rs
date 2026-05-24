use crate::{
    ResourceRefType,
    consts::{INDEX_FILENAME, PAGES_FOLDER, PUBLIC_FOLDER},
    resources::ResourceManager,
    routing::{MethodRouter, get},
    vfs::VfsPath,
};
use log::{debug, error, info, trace, warn};
use notify::{
    Event, EventKind, RecursiveMode, Watcher,
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum ContentEventKind {
    NewPublic,
    NewPage,
    RemovePublic,
    RemovePage,
}

impl ContentEventKind {
    pub fn is_public(&self) -> bool {
        matches!(self, ContentEventKind::NewPublic) || matches!(self, ContentEventKind::RemovePublic)
    }

    // pub fn is_pages(&self) -> bool {
    //     matches!(self, ContentEventKind::NewPage) || matches!(self, ContentEventKind::RemovePage)
    // }
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

pub type FilesListener = crossbeam_channel::Receiver<notify::Result<Event>>;

pub struct FilesWatcher {
    _watcher: Arc<dyn Watcher + Send + Sync>,
    listener: FilesListener,
    root: PathBuf,
}

impl Iterator for &mut FilesWatcher {
    type Item = ContentEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.listener.recv().ok()?.ok()?;
        let event = notify_event_to_content_event(&self.root, event);

        if event.is_none() { None } else { Some(event) }
    }
}

fn notify_event_to_content_event(root: &Path, e: Event) -> ContentEvent {
    let Some(path) = e.paths.into_iter().filter_map(|p| real_path_to_vfs(root, &p)).next()
    else {
        warn!("none path in filesystem event");
        return ContentEvent::None;
    };

    match e.kind {
        EventKind::Create(kind) => {
            trace!("file {path} created, kind: {kind:?}");

            match kind {
                CreateKind::Any => {
                    if is_public_folder(&path) {
                        ContentEvent::Change {
                            kind: ContentEventKind::NewPublic,
                            path,
                        }
                    }
                    else if is_pages_folder(&path) {
                        ContentEvent::Change {
                            kind: ContentEventKind::NewPage,
                            path,
                        }
                    }
                    else {
                        ContentEvent::None
                    }
                }
                _ => ContentEvent::None,
            }
        }
        EventKind::Remove(kind) => {
            trace!("file {path} removed, kind: {kind:?}");

            if let RemoveKind::Any = kind {
                if is_public_folder(&path) {
                    ContentEvent::Change {
                        kind: ContentEventKind::RemovePublic,
                        path,
                    }
                }
                else if is_pages_folder(&path) {
                    ContentEvent::Change {
                        kind: ContentEventKind::RemovePage,
                        path,
                    }
                }
                else {
                    ContentEvent::None
                }
            }
            else {
                ContentEvent::None
            }
        }
        EventKind::Modify(kind) => {
            debug!("file {path} modified, kind: {kind:?}");
            match kind {
                ModifyKind::Name(rename_mode) => {
                    debug!("rename mode: {rename_mode:?}");
                    match rename_mode {
                        RenameMode::From => {
                            if is_public_folder(&path) {
                                ContentEvent::Change {
                                    kind: ContentEventKind::RemovePublic,
                                    path,
                                }
                            }
                            else if is_pages_folder(&path) {
                                ContentEvent::Change {
                                    kind: ContentEventKind::RemovePage,
                                    path,
                                }
                            }
                            else {
                                ContentEvent::None
                            }
                        }
                        RenameMode::To => {
                            if is_public_folder(&path) {
                                ContentEvent::Change {
                                    kind: ContentEventKind::NewPublic,
                                    path,
                                }
                            }
                            else if is_pages_folder(&path) {
                                ContentEvent::Change {
                                    kind: ContentEventKind::NewPage,
                                    path,
                                }
                            }
                            else {
                                ContentEvent::None
                            }
                        }
                        _ => ContentEvent::None,
                    }
                }
                _ => ContentEvent::None,
            }
        }
        _ => ContentEvent::None,
    }
}

fn is_public_folder(path: &VfsPath) -> bool {
    path.starts_with(&VfsPath::new(PUBLIC_FOLDER))
}

fn is_pages_folder(path: &VfsPath) -> bool {
    path.starts_with(&VfsPath::new(PAGES_FOLDER)) && path.filename() == INDEX_FILENAME
}

fn real_path_to_vfs(root: &Path, path: &Path) -> Option<VfsPath> {
    path.strip_prefix(root).map(|p| VfsPath::new(p.to_string_lossy().replace('\\', "/"))).ok()
}

pub fn init_watcher(content: &Path, router: Arc<RwLock<MethodRouter>>, resources: Arc<RwLock<ResourceManager>>) {
    let (tx, rx) = crossbeam_channel::unbounded();

    let mut watcher = match notify::recommended_watcher(tx) {
        Ok(w) => w,
        Err(err) => {
            error!("failed to initialize files watcher because {err}, changes to files will not be applied until the server is restarted");
            return;
        }
    };

    match watcher.watch(content, RecursiveMode::Recursive) {
        Ok(_) => {
            info!("watcher initialized at {}", content.display());
            let watcher = FilesWatcher {
                root: content.to_owned(),
                _watcher: Arc::new(watcher),
                listener: rx,
            };

            tokio::spawn(async move {
                watch_content(watcher, router, resources).await;
            });
        }
        Err(err) => {
            error!("failed to initialize watcher on {} because {err}", content.display());
        }
    }
}

async fn watch_content(mut watcher: FilesWatcher, router: Arc<RwLock<MethodRouter>>, resources: Arc<RwLock<ResourceManager>>) {
    loop {
        for event in &mut watcher {
            let mut router = router.write().await;

            if let ContentEvent::Change { path, kind } = event {
                let web_path = if kind.is_public() {
                    path.strip_prefix(VfsPath::new(PUBLIC_FOLDER)).unwrap()
                }
                else {
                    path.strip_prefix(VfsPath::new(PAGES_FOLDER)).unwrap()
                };

                match kind {
                    ContentEventKind::NewPublic => {
                        if let Err(err) = router.try_add(get(&web_path), ResourceRefType::File(path)) {
                            error!("unable to update router with route {web_path} because {err}");
                        }
                    }
                    ContentEventKind::NewPage => resources.read().await.load_page(&mut router, path).await,
                    ContentEventKind::RemovePublic | ContentEventKind::RemovePage => router.remove(get(&web_path)),
                }
            }
        }
    }
}
