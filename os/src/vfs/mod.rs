mod device;
mod fs;
mod err;
mod structs;
mod paths;

#[macro_use]
mod macros;

use device::DISK_DEVICE;
use fs::init_rootfs;

pub fn init_rootfs_on_disk() {
    init_rootfs(&DISK_DEVICE);
}

use alloc::{sync::Arc, vec::Vec};
pub use device::{BlockDevice, disk_device_test};
use err::{DevError, DevResult};
pub use paths::test_path_canonicalize;

pub use fs::fs_test;

pub use fs::{list_dir_by_str, read_file_by_str, get_file_size};

pub use self::structs::{FileSystemInfo, VfsDirEntry, VfsNodeAttr, VfsNodePerm, VfsNodeType};

/// A wrapper of [`Arc<dyn VfsNodeOps>`].
pub type VfsNodeRef = Arc<dyn VfsNodeOps>;

/// Alias of [`DevError`].
pub type VfsError = DevError;

/// Alias of [`DevResult`].
// pub type DevResult<T = ()> = DevResult<T>;

/// Filesystem operations.
pub trait VfsOps: Send + Sync {
    
    #[allow(unused)]
    /// Do something when the filesystem is mounted.
    fn mount(&self, _path: &str, _mount_point: VfsNodeRef) -> DevResult {
        Ok(())
    }

    /// Do something when the filesystem is unmounted.
    fn umount(&self) -> DevResult {
        Ok(())
    }

    #[allow(unused)]
    /// Format the filesystem.
    fn format(&self) -> DevResult {
        yy_err!(Unsupported)
    }

    #[allow(unused)]
    /// Get the attributes of the filesystem.
    fn statfs(&self) -> DevResult<FileSystemInfo> {
        yy_err!(Unsupported)
    }

    /// Get the root directory of the filesystem.
    fn root_dir(&self) -> VfsNodeRef;
}

/// Node (file/directory) operations.
pub trait VfsNodeOps: Send + Sync {
    
    #[allow(unused)]
    /// Do something when the node is opened.
    fn open(&self) -> DevResult {
        Ok(())
    }

    #[allow(unused)]
    /// Do something when the node is closed.
    fn release(&self) -> DevResult {
        Ok(())
    }

    #[allow(unused)]
    /// Get the attributes of the node.
    fn get_attr(&self) -> DevResult<VfsNodeAttr> {
        yy_err!(Unsupported)
    }

    // file operations:

    #[allow(unused)]
    /// Read data from the file at the given offset.
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> DevResult<usize> {
        yy_err!(InvalidInput)
    }

    #[allow(unused)]
    /// Write data to the file at the given offset.
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> DevResult<usize> {
        yy_err!(InvalidInput)
    }

    #[allow(unused)]
    /// Flush the file, synchronize the data to disk.
    fn fsync(&self) -> DevResult {
        yy_err!(InvalidInput)
    }

    #[allow(unused)]
    /// Truncate the file to the given size.
    fn truncate(&self, _size: u64) -> DevResult {
        yy_err!(InvalidInput)
    }

    // directory operations:

    #[allow(unused)]
    /// Get the parent directory of this directory.
    ///
    /// Return `None` if the node is a file.
    fn parent(&self) -> Option<VfsNodeRef> {
        None
    }

    /// Lookup the node with given `path` in the directory.
    ///
    /// Return the node if found.
    fn lookup(self: Arc<Self>, _path: &str) -> DevResult<VfsNodeRef> {
        yy_err!(Unsupported)
    }

    /// Create a new node with the given `path` in the directory
    ///
    /// Return [`Ok(())`](Ok) if it already exists.
    fn create(&self, _path: &str, _ty: VfsNodeType) -> DevResult {
        yy_err!(Unsupported)
    }

    /// Remove the node with the given `path` in the directory.
    fn remove(&self, _path: &str) -> DevResult {
        yy_err!(Unsupported)
    }

    #[allow(unused)]
    /// Read directory entries into `dirents`, starting from `start_idx`.
    fn read_dir(&self) -> DevResult<Vec<VfsDirEntry>> {
        yy_err!(Unsupported)
    }

    /// Renames or moves existing file or directory.
    fn rename(&self, _src_path: &str, _dst_path: &str) -> DevResult {
        yy_err!(Unsupported)
    }

    #[allow(unused)]
    /// Convert `&self` to [`&dyn Any`][1] that can use
    /// [`Any::downcast_ref`][2].
    ///
    /// [1]: core::any::Any
    /// [2]: core::any::Any#method.downcast_ref
    fn as_any(&self) -> &dyn core::any::Any {
        unimplemented!()
    }
}
