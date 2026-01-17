use serde::Serialize;
use std::{
    env,
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
};
use vfs::async_vfs::{AsyncOverlayFS, AsyncPhysicalFS, AsyncVfsPath};
#[cfg(not(debug_assertions))]
use {
    futures_util::{
        AsyncWrite, Stream,
        io::{BufReader, Cursor},
        stream,
    },
    rust_embed::Embed,
    std::time::{Duration, UNIX_EPOCH},
    tree_ds::prelude::{Node, Tree},
    vfs::{
        VfsError, VfsFileType, VfsMetadata, VfsResult,
        async_vfs::{AsyncFileSystem, SeekAndRead},
        error::VfsErrorKind,
    },
};

pub type VirtualFS = Arc<AsyncOverlayFS>;

#[cfg(not(debug_assertions))]
pub async fn init_vfs(root: &Path) -> VirtualFS {
    tokio::fs::create_dir_all(root).await.expect("unable to create vfs dir");
    let embed_fs = AsyncEmbedFS::new();
    let physical_fs = AsyncPhysicalFS::new(root);
    Arc::new(AsyncOverlayFS::new(&[AsyncVfsPath::new(physical_fs), AsyncVfsPath::new(embed_fs)]))
}

#[cfg(debug_assertions)]
pub async fn init_vfs(root: &Path) -> VirtualFS {
    let target_content = Path::new(&env::current_exe().unwrap().parent().unwrap()).join("content");
    tokio::fs::create_dir_all(&target_content)
        .await
        .expect("unable to create target content dir");

    let target_content = AsyncPhysicalFS::new(target_content);
    let embed_content = AsyncPhysicalFS::new(root);
    Arc::new(AsyncOverlayFS::new(&[
        AsyncVfsPath::new(target_content),
        AsyncVfsPath::new(embed_content),
    ]))
}

#[derive(Clone, Debug)]
pub struct PageLayout(Arc<VfsPath>);

impl PageLayout {
    pub fn new(path: VfsPath) -> Self {
        Self(Arc::new(path))
    }
}

impl Deref for PageLayout {
    type Target = VfsPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for PageLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct PageDir {
    pub path: VfsPath,
    pub inherited_layouts: Vec<PageLayout>,
}

impl PageDir {
    pub fn new(path: VfsPath) -> Self {
        Self {
            path,
            inherited_layouts: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct VfsPath(String);

impl VfsPath {
    pub fn new(path: impl ToString) -> Self {
        let path = path.to_string();
        let normalized = path.trim_matches('/');
        if !normalized.is_empty() {
            Self("/".to_owned() + normalized)
        }
        else {
            Self(normalized.to_owned())
        }
    }

    pub fn strip_prefix(&self, prefix: VfsPath) -> Option<Self> {
        self.0.strip_prefix(&*prefix).map(|prefix| Self(prefix.to_owned()))
    }

    pub fn starts_with(&self, start: &VfsPath) -> bool {
        self.0.starts_with(&**start)
    }

    pub fn filename(&self) -> String {
        self.0.split('/').next_back().unwrap().to_owned()
    }

    pub fn parent(&self) -> Option<Self> {
        let components = self.0.split('/').collect::<Vec<_>>();
        let count = components.len() - 1;
        Some(Self::new(components.into_iter().take(count).collect::<Vec<_>>().join("/")))
    }

    pub fn join(&self, path: impl Into<VfsPath>) -> Self {
        Self(self.0.clone() + "/" + &path.into())
    }

    pub fn as_string(self) -> String {
        self.0
    }
}

impl From<String> for VfsPath {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for VfsPath {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl<const N: usize> From<[VfsPath; N]> for VfsPath {
    fn from(value: [VfsPath; N]) -> Self {
        Self(
            value
                .into_iter()
                .fold(String::new(), |acc, elem| if acc.is_empty() { elem.0 } else { acc + "/" + &elem }),
        )
    }
}

impl Deref for VfsPath {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VfsPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for VfsPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Path> for VfsPath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[cfg(not(debug_assertions))]
#[derive(Embed)]
#[folder = "../content"]
struct Content;

#[cfg(not(debug_assertions))]
#[derive(Clone, PartialEq, Eq, Debug)]
struct DirEntry {
    path: String,
    is_dir: bool,
}

#[cfg(not(debug_assertions))]
#[derive(Debug)]
struct AsyncEmbedFS {
    tree: FilesTree,
}

#[cfg(not(debug_assertions))]
impl AsyncEmbedFS {
    pub fn new() -> Self {
        let mut tree = FilesTree::new();

        for file in Content::iter() {
            tree.add_file(file.as_ref()).expect("failed to add file to tree");
        }

        Self { tree }
    }
}

#[cfg(not(debug_assertions))]
#[async_trait::async_trait]
impl AsyncFileSystem for AsyncEmbedFS {
    async fn read_dir(&self, path: &str) -> VfsResult<Box<dyn Unpin + Stream<Item = String> + Send>> {
        let dir = path.trim_matches('/').to_owned();

        if dir == ".whiteout" {
            return Ok(Box::new(stream::empty()));
        }

        let Some(file) = self.tree.get_meta(dir.clone())
        else {
            return Err(VfsErrorKind::FileNotFound.into());
        };

        if !file.is_dir {
            return Err(VfsErrorKind::Other("not a directory".to_string()).into());
        }

        if let Some(entries) = self.tree.read_dir(dir.clone()) {
            Ok(Box::new(stream::iter(entries.into_iter().map(|e| e.path))))
        }
        else {
            Err(VfsError::from(VfsErrorKind::Other("not a directory".to_string())))
        }
    }

    async fn create_dir(&self, _path: &str) -> VfsResult<()> {
        unimplemented!("embed fs does not support creating directories")
    }

    async fn open_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndRead + Send + Unpin>> {
        let meta = self.tree.get_meta(path.trim_matches('/').to_owned()).ok_or(VfsErrorKind::FileNotFound)?;

        let content = Content::get(&meta.path)
            .map(|c| c.data.to_vec())
            .ok_or(VfsError::from(VfsErrorKind::FileNotFound))?;

        let cursor = Cursor::new(content);
        let buf = BufReader::new(cursor);
        Ok(Box::new(buf))
    }

    async fn create_file(&self, _path: &str) -> VfsResult<Box<dyn AsyncWrite + Send + Unpin>> {
        unimplemented!("embed fs does not support creating files")
    }

    async fn append_file(&self, _path: &str) -> VfsResult<Box<dyn AsyncWrite + Send + Unpin>> {
        unimplemented!("embed fs does not support appending files")
    }

    async fn metadata(&self, path: &str) -> VfsResult<VfsMetadata> {
        let path = path.trim_matches('/').to_owned();
        match path.as_str() {
            ".whiteout" => Ok(VfsMetadata {
                file_type: VfsFileType::Directory,
                len: 0,
                created: None,
                modified: None,
                accessed: None,
            }),
            _ => {
                let meta = self.tree.get_meta(path.clone()).ok_or(VfsError::from(VfsErrorKind::FileNotFound))?;

                if meta.is_dir {
                    Ok(VfsMetadata {
                        file_type: VfsFileType::Directory,
                        len: 0,
                        created: None,
                        modified: None,
                        accessed: None,
                    })
                }
                else {
                    Content::get(&meta.path)
                        .map(|c| {
                            let last_modified = c
                                .metadata
                                .last_modified()
                                .map(|t| UNIX_EPOCH.checked_add(Duration::from_secs(t)).unwrap());

                            VfsMetadata {
                                file_type: if meta.is_dir { VfsFileType::Directory } else { VfsFileType::File },
                                len: c.data.len() as u64,
                                created: c.metadata.created().map(|t| UNIX_EPOCH.checked_add(Duration::from_secs(t)).unwrap()),
                                modified: last_modified,
                                accessed: last_modified,
                            }
                        })
                        .ok_or(VfsError::from(VfsErrorKind::FileNotFound))
                }
            }
        }
    }

    async fn exists(&self, path: &str) -> VfsResult<bool> {
        let path = path.trim_matches('/');
        match path {
            "" | ".whiteout" => Ok(true),
            &_ => {
                let Some(entry) = self.tree.get_meta(path.into())
                else {
                    return Ok(false);
                };

                if entry.is_dir {
                    Ok(true)
                }
                else {
                    Ok(Content::get(&entry.path).is_some())
                }
            }
        }
    }

    async fn remove_file(&self, _path: &str) -> VfsResult<()> {
        unimplemented!("embed fs does not support removing files")
    }

    async fn remove_dir(&self, _path: &str) -> VfsResult<()> {
        unimplemented!("embed fs does not support removing directories")
    }
}

#[cfg(not(debug_assertions))]
#[derive(Debug)]
struct FilesTree(Tree<String, DirEntry>);

#[cfg(not(debug_assertions))]
impl FilesTree {
    pub fn new() -> Self {
        let mut tree = Tree::new(None);
        let root = Node::new(
            "".to_owned(),
            Some(DirEntry {
                path: "".to_owned(),
                is_dir: true,
            }),
        );
        tree.add_node(root, None).expect("failed to add root node");
        Self(tree)
    }

    pub fn add_file(&mut self, path: &str) -> tree_ds::prelude::Result<()> {
        let components = path.split('/').collect::<Vec<_>>();

        let mut stack = vec![self.0.get_root_node().unwrap()];
        for (i, _) in components[..components.len()].iter().enumerate() {
            let id = components[..i].join("/");
            if id.is_empty() {
                continue;
            }

            let parent = stack.last().unwrap();
            let node = Node::new(id.clone(), Some(DirEntry { path: id, is_dir: true }));

            self.0.add_node(node.clone(), Some(&parent.get_node_id()?))?;
            stack.push(node);
        }

        let id = components.join("/");
        let node = Node::new(id.clone(), Some(DirEntry { path: id, is_dir: false }));

        let parent = stack.pop().unwrap();
        self.0.add_node(node, Some(&parent.get_node_id()?))?;

        Ok(())
    }

    pub fn get_meta(&self, path: String) -> Option<DirEntry> {
        self.0.get_node_by_id(&path)?.get_value().ok()?
    }

    pub fn read_dir(&self, path: String) -> Option<Vec<DirEntry>> {
        Some(
            self.0
                .get_node_by_id(&path)?
                .get_children_ids()
                .ok()?
                .into_iter()
                .filter_map(|id| self.0.get_node_by_id(&id)?.get_value().ok()?)
                .collect::<Vec<_>>(),
        )
    }
}
