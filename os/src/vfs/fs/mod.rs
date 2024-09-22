//! Root directory of the filesystem
//!
//! TODO: it doesn't work very well if the mount points have containment relationships.

mod fat;
mod ext4;

use core::assert_matches::assert_matches;

use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use alloc::vec;
use fat::FatFileSystem;
use ext4::Ext4FileSystem;
use super::err::{DevError, DevResult};
use super::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps};
use spin::Mutex;
use crate::sync::LazyInit;

use crate::{impl_vfs_dir_default, yy_err};

static CURRENT_DIR_PATH: Mutex<String> = Mutex::new(String::new());
static CURRENT_DIR: LazyInit<Mutex<VfsNodeRef>> = LazyInit::new();

struct MountPoint {
    path: &'static str,
    fs: Arc<dyn VfsOps>,
}

struct RootDirectory {
    main_fs: Arc<dyn VfsOps>,
    mounts: Vec<MountPoint>,
}

static ROOT_DIR: LazyInit<Arc<RootDirectory>> = LazyInit::new();

impl MountPoint {
    pub fn new(path: &'static str, fs: Arc<dyn VfsOps>) -> Self {
        Self { path, fs }
    }
}

impl Drop for MountPoint {
    fn drop(&mut self) {
        self.fs.umount().ok();
    }
}

impl RootDirectory {
    pub const fn new(main_fs: Arc<dyn VfsOps>) -> Self {
        Self {
            main_fs,
            mounts: Vec::new(),
        }
    }

    #[allow(unused)]
    pub fn mount(&mut self, path: &'static str, fs: Arc<dyn VfsOps>) -> DevResult {
        if path == "/" {
            return yy_err!(InvalidInput, "cannot mount root filesystem");
        }
        if !path.starts_with('/') {
            return yy_err!(InvalidInput, "mount path must start with '/'");
        }
        if self.mounts.iter().any(|mp| mp.path == path) {
            return yy_err!(InvalidInput, "mount point already exists");
        }
        // create the mount point in the main filesystem if it does not exist
        self.main_fs.root_dir().create(path, VfsNodeType::Dir)?;
        fs.mount(path, self.main_fs.root_dir().lookup(path)?)?;
        self.mounts.push(MountPoint::new(path, fs));
        Ok(())
    }

    pub fn _umount(&mut self, path: &str) {
        self.mounts.retain(|mp| mp.path != path);
    }

    pub fn contains(&self, path: &str) -> bool {
        self.mounts.iter().any(|mp| mp.path == path)
    }

    fn lookup_mounted_fs<F, T>(&self, path: &str, f: F) -> DevResult<T>
    where
        F: FnOnce(Arc<dyn VfsOps>, &str) -> DevResult<T>,
    {
        debug!("lookup at root: {}", path);
        let path = path.trim_matches('/');
        if let Some(rest) = path.strip_prefix("./") {
            return self.lookup_mounted_fs(rest, f);
        }

        let mut idx = 0;
        let mut max_len = 0;

        // Find the filesystem that has the longest mounted path match
        // TODO: more efficient, e.g. trie

        for (i, mp) in self.mounts.iter().enumerate() {
            // skip the first '/'
            // two conditions
            // 1. path == mp.path, e.g. dev
            // 2. path == mp.path + '/', e.g. dev/
            let prev = mp.path[1..].to_string() + "/";
            if path.starts_with(&mp.path[1..])
                && (path.len() == prev.len() - 1 || path.starts_with(&prev))
                && prev.len() > max_len
            {
                max_len = mp.path.len() - 1;
                idx = i;
            }
        }
        if max_len == 0 {
            f(self.main_fs.clone(), path) // not matched any mount point
        } else {
            f(self.mounts[idx].fs.clone(), &path[max_len..]) // matched at `idx`
        }
    }
}

impl VfsNodeOps for RootDirectory {
    impl_vfs_dir_default! {}

    fn get_attr(&self) -> DevResult<VfsNodeAttr> {
        self.main_fs.root_dir().get_attr()
    }

    fn lookup(self: Arc<Self>, path: &str) -> DevResult<VfsNodeRef> {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            let dir = fs.root_dir();
            dir.lookup(rest_path)
        })
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> DevResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                Ok(()) // already exists
            } else {
                fs.root_dir().create(rest_path, ty)
            }
        })
    }

    fn remove(&self, path: &str) -> DevResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                yy_err!(PermissionDenied) // cannot remove mount points
            } else {
                fs.root_dir().remove(rest_path)
            }
        })
    }

    fn rename(&self, src_path: &str, dst_path: &str) -> DevResult {
        self.lookup_mounted_fs(src_path, |fs, rest_path| {
            if rest_path.is_empty() {
                yy_err!(PermissionDenied) // cannot rename mount points
            } else {
                fs.root_dir().rename(rest_path, dst_path)
            }
        })
    }
}

pub fn init_rootfs(disk: &crate::vfs::device::Disk) {
    #[cfg(not(feature = "ext4"))]
    let use_fatfs = true; // TODO: detect file system type from disk
    #[cfg(feature = "ext4")]
    let use_fatfs = false; // TODO: detect file system type from disk
    let main_fs: Arc<dyn VfsOps> = if use_fatfs{
        static FAT_FS: LazyInit<Arc<FatFileSystem>> = LazyInit::new();
        FAT_FS.init_by(Arc::new(FatFileSystem::new(disk.clone())));
        FAT_FS.init();
        FAT_FS.clone()
    } else {
        static EXT4_FS: LazyInit<Arc<Ext4FileSystem>> = LazyInit::new();
        EXT4_FS.init_by(Arc::new(Ext4FileSystem::new(disk.clone())));
        // EXT4_FS.init();
        EXT4_FS.clone()
    };

    let root_dir = RootDirectory::new(main_fs);

    ROOT_DIR.init_by(Arc::new(root_dir));
    CURRENT_DIR.init_by(Mutex::new(ROOT_DIR.clone()));
    *CURRENT_DIR_PATH.lock() = "/".into();
}

fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef {
    if path.starts_with('/') {
        ROOT_DIR.clone()
    } else {
        dir.cloned().unwrap_or_else(|| CURRENT_DIR.lock().clone())
    }
}

pub fn absolute_path(path: &str) -> DevResult<String> {
    if path.starts_with('/') {
        Ok(crate::vfs::paths::canonicalize(path))
    } else {
        let path = CURRENT_DIR_PATH.lock().clone() + path;
        Ok(crate::vfs::paths::canonicalize(&path))
    }
}

pub fn lookup(dir: Option<&VfsNodeRef>, path: &str) -> DevResult<VfsNodeRef> {
    if path.is_empty() {
        return yy_err!(NotFound);
    }
    let node = parent_node_of(dir, path).lookup(path)?;
    if path.ends_with('/') && !node.get_attr()?.is_dir() {
        yy_err!(NotADirectory)
    } else {
        Ok(node)
    }
}

pub fn create_file(dir: Option<&VfsNodeRef>, path: &str) -> DevResult<VfsNodeRef> {
    if path.is_empty() {
        return yy_err!(NotFound);
    } else if path.ends_with('/') {
        return yy_err!(NotADirectory);
    }
    let parent = parent_node_of(dir, path);
    parent.create(path, VfsNodeType::File)?;
    parent.lookup(path)
}

pub fn create_file_by_str(dir: &str, path: &str) -> DevResult {
    let root = &(ROOT_DIR.as_ref().main_fs.root_dir());
    let node = match lookup(Some(root), dir) {
        Ok(node) => node, // 或者根据你的逻辑返回其他值
        Err(e) => return Err(e),
    };
    let dir = Some(&node);
    match create_file(dir, path) {
        Ok(_) => Ok(()),
        Err(DevError::AlreadyExists) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn create_dir(dir: Option<&VfsNodeRef>, path: &str) -> DevResult {
    match lookup(dir, path) {
        Ok(_) => yy_err!(AlreadyExists),
        Err(DevError::NotFound) => {
            // println!("create_dir error: NotFound");
            parent_node_of(dir, path).create(path, VfsNodeType::Dir)
        },
        Err(e) => {
            Err(e)
        },
    }
}

pub fn create_dir_by_str(dir: &str, path: &str) -> DevResult {
    println!("create_dir_by_str: dir: {}, path: {}", dir, path);
    let root = &(ROOT_DIR.as_ref().main_fs.root_dir());
    let node = match lookup(Some(root), dir) {
        Ok(node) => node, // 或者根据你的逻辑返回其他值
        Err(e) => {
            println!("create_dir_by_str error: lookup error: {:?}", e);
            return Err(e)
        },
    };
    let dir = Some(&node);
    create_dir(dir, path)
}

pub fn remove_file(dir: Option<&VfsNodeRef>, path: &str) -> DevResult {
    let node = lookup(dir, path)?;
    let attr = node.get_attr()?;
    if attr.is_dir() {
        yy_err!(IsADirectory)
    } else if !attr.perm().owner_writable() {
        yy_err!(PermissionDenied)
    } else {
        parent_node_of(dir, path).remove(path)
    }
}

pub fn remove_file_by_str(dir: &str, path: &str) -> DevResult {
    let root = &(ROOT_DIR.as_ref().main_fs.root_dir());
    let node = match lookup(Some(root), dir) {
        Ok(node) => node, // 或者根据你的逻辑返回其他值
        Err(e) => return Err(e),
    };
    let dir = Some(&node);
    remove_file(dir, path)
}

pub fn remove_dir(dir: Option<&VfsNodeRef>, path: &str) -> DevResult {
    if path.is_empty() {
        return yy_err!(NotFound);
    }
    let path_check = path.trim_matches('/');
    if path_check.is_empty() {
        return yy_err!(DirectoryNotEmpty); // rm -d '/'
    } else if path_check == "."
        || path_check == ".."
        || path_check.ends_with("/.")
        || path_check.ends_with("/..")
    {
        return yy_err!(InvalidInput);
    }
    if ROOT_DIR.contains(&absolute_path(path)?) {
        return yy_err!(PermissionDenied);
    }

    let node = lookup(dir, path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        yy_err!(NotADirectory)
    } else if !attr.perm().owner_writable() {
        yy_err!(PermissionDenied)
    } else {
        parent_node_of(dir, path).remove(path)
    }
}

pub fn remove_dir_by_str(dir: &str, path: &str) -> DevResult {
    let root = &(ROOT_DIR.as_ref().main_fs.root_dir());
    let node = match lookup(Some(root), dir) {
        Ok(node) => node, // 或者根据你的逻辑返回其他值
        Err(e) => return Err(e),
    };
    let dir = Some(&node);
    remove_dir(dir, path)
}

pub fn open_file(dir: Option<&VfsNodeRef>, path: &str) -> DevResult<VfsNodeRef> {
    let node = lookup(dir, path)?;
    let attr = node.get_attr()?;
    if !attr.is_file() {
        yy_err!(NotAFile)
    } else if !attr.perm().owner_readable() {
        yy_err!(PermissionDenied)
    } else {
        Ok(node)
    }
}

pub fn open_file_by_str(dir: &str, path: &str) -> DevResult<VfsNodeRef> {
    let root = &(ROOT_DIR.as_ref().main_fs.root_dir());
    let node = match lookup(Some(root), dir) {
        Ok(node) => node, // 或者根据你的逻辑返回其他值
        Err(e) => return Err(e),
    };
    let dir = Some(&node);
    open_file(dir, path)
}

pub fn read_file(node: &VfsNodeRef, offset: usize, size: usize) -> DevResult<Vec<u8>> {
    let attr = node.get_attr()?;
    if !attr.is_file() {
        yy_err!(NotAFile)
    } else if !attr.perm().owner_readable() {
        yy_err!(PermissionDenied)
    } else {
        let mut buffer = vec![0u8; size];
        match node.read_at(offset as u64, buffer.as_mut_slice()) {
            Ok(_) => Ok(buffer),
            Err(e) => Err(e)
        }
    }
}

pub fn write_file(node: &VfsNodeRef, offset: usize, data: &[u8]) -> DevResult<usize> {
    let attr = node.get_attr()?;
    if !attr.is_file() {
        yy_err!(NotAFile)
    } else if !attr.perm().owner_writable() {
        yy_err!(PermissionDenied)
    } else {
        match node.write_at(offset as u64, data) {
            Ok(size) => Ok(size),
            Err(e) => Err(e)
        }
    }
}

pub fn read_file_by_str(path: &str, offset: usize, size: usize) -> DevResult<Vec<u8>> {
    let node = open_file_by_str(path, path)?;
    read_file(&node, offset, size)
}

pub fn write_file_by_str(path: &str, offset: usize, data: &[u8]) -> DevResult<usize> {
    let node = open_file_by_str(path, path)?;
    write_file(&node, offset, data)
}

pub fn current_dir() -> DevResult<String> {
    Ok(CURRENT_DIR_PATH.lock().clone())
}

pub fn set_current_dir(path: &str) -> DevResult {
    let mut abs_path = absolute_path(path)?;
    if !abs_path.ends_with('/') {
        abs_path += "/";
    }
    if abs_path == "/" {
        *CURRENT_DIR.lock() = ROOT_DIR.clone();
        *CURRENT_DIR_PATH.lock() = "/".into();
        return Ok(());
    }

    let node = lookup(None, &abs_path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        yy_err!(NotADirectory)
    } else if !attr.perm().owner_executable() {
        yy_err!(PermissionDenied)
    } else {
        *CURRENT_DIR.lock() = node;
        *CURRENT_DIR_PATH.lock() = abs_path;
        Ok(())
    }
}

pub fn rename(old: &str, new: &str) -> DevResult {
    if parent_node_of(None, new).lookup(new).is_ok() {
        warn!("dst file already exist, now remove it");
        remove_file(None, new)?;
    }
    parent_node_of(None, old).rename(old, new)
}

pub fn list_dir(dir: Option<&VfsNodeRef>, path: &str) -> DevResult<Vec<String>> {
    let node = lookup(dir, path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        yy_err!(NotADirectory)
    } else if !attr.perm().owner_readable() {
        yy_err!(PermissionDenied)
    } else {
        let mut entries = Vec::new();
        for entry in node.read_dir()? {
            let name = entry.name_as_str();
            if name == "." || name == ".." {
                continue;
            }
            entries.push(name.to_string());
        }
        Ok(entries)
    }
}

pub fn list_dir_by_str(dir: &str, path: &str) -> DevResult<Vec<String>> {
    let root = &(ROOT_DIR.as_ref().main_fs.root_dir());
    let node = match lookup(Some(root), dir) {
        Ok(node) => node, // 或者根据你的逻辑返回其他值
        Err(e) => return Err(e),
    };
    let dir = Some(&node);
    list_dir(dir, path)
}

pub fn get_file_size(path: &str) -> DevResult<u64> {
    let node = open_file_by_str(path, path)?;
    let attr = node.get_attr()?;
    if !attr.is_file() {
        yy_err!(NotAFile)
    } else {
        Ok(attr.size())
    }
}

pub fn fs_test() {
    assert_matches!(create_dir_by_str("/", "yes"), Ok(()));
    assert_matches!(create_file_by_str("/", "/yes/no"), Ok(_));
    assert_matches!(create_dir_by_str("/", "/yes/yes"), Ok(()));
    assert_matches!(rename("/yes/no", "/yes/no2"), Ok(()));
    println!("Current dir: {}", current_dir().unwrap());
    assert_matches!(set_current_dir("/yes/yes"), Ok(()));
    println!("Current dir changed to: {}", current_dir().unwrap());
    assert_matches!(create_file_by_str("/", "no2"), Ok(_));
    assert_matches!(remove_file_by_str("/", "no2"), Ok(()));
    assert_matches!(remove_dir_by_str("/", "/yes/yes"), Ok(()));
    let bytes = b"Hello World in FAT32!\n";
    let bytes_len = bytes.len();
    assert_matches!(write_file_by_str("/yes/no2", 0, bytes), Ok(_));
    assert_eq!(read_file_by_str("/yes/no2", 0, bytes_len).unwrap(), bytes);
    // List dir
    println!("List dir: {:?}", list_dir_by_str("/", "/bin/").unwrap());
    // Get file size
    assert_eq!(get_file_size("/yes/no2").unwrap(), bytes_len as u64);
    println!("fs test passed");
}

