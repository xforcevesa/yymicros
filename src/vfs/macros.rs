/// When implement [`VfsNodeOps`] on a directory node, add dummy file operations
/// that just return an error.
///
/// [`VfsNodeOps`]: crate::VfsNodeOps
#[macro_export]
macro_rules! impl_vfs_dir_default {
    () => {
        fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> $crate::vfs::err::DevResult<usize> {
            $crate::yy_err!(IsADirectory)
        }

        fn write_at(&self, _offset: u64, _buf: &[u8]) -> $crate::vfs::err::DevResult<usize> {
            $crate::yy_err!(IsADirectory)
        }

        fn fsync(&self) -> $crate::vfs::err::DevResult {
            $crate::yy_err!(IsADirectory)
        }

        fn truncate(&self, _size: u64) -> $crate::vfs::err::DevResult {
            $crate::yy_err!(IsADirectory)
        }

        #[inline]
        fn as_any(&self) -> &dyn core::any::Any {
            self
        }
    };
}

/// When implement [`VfsNodeOps`] on a non-directory node, add dummy directory
/// operations that just return an error.
///
/// [`VfsNodeOps`]: crate::VfsNodeOps
#[macro_export]
macro_rules! impl_vfs_non_dir_default {
    () => {
        fn lookup(
            self: alloc::sync::Arc<Self>,
            _path: &str,
        ) -> $crate::vfs::err::DevResult<super::VfsNodeRef> {
            $crate::yy_err!(NotADirectory)
        }

        fn create(&self, _path: &str, _ty: super::VfsNodeType) -> $crate::vfs::err::DevResult {
            $crate::yy_err!(NotADirectory)
        }

        fn remove(&self, _path: &str) -> $crate::vfs::err::DevResult {
            $crate::yy_err!(NotADirectory)
        }

        fn read_dir(
            &self,
            _start_idx: usize,
            _dirents: &mut [super::VfsDirEntry],
        ) -> super::DevResult<usize> {
            $crate::yy_err!(NotADirectory)
        }

        #[inline]
        fn as_any(&self) -> &dyn core::any::Any {
            self
        }
    };
}

/// Raise a VFS error with the given kind.
#[macro_export]
macro_rules! yy_err {
    (
        InvalidInput
    ) => (
        {
            info!("VFS Error: {}", stringify!($kind));
            $crate::vfs::err::DevResult::Err($crate::vfs::err::DevError::InvalidInput(None))
        }
    );
    (
        $kind:ident
    ) => (
        {
            info!("VFS Error: {}", stringify!($kind));
            $crate::vfs::err::DevResult::Err($crate::vfs::err::DevError::$kind)
        }
    );
    (
        $kind:ident,
        $msg:expr
    ) => (
        {
            info!("VFS Error: {}: {}", stringify!($kind), $msg);
            $crate::vfs::err::DevResult::Err($crate::vfs::err::DevError::$kind(Some($msg.to_string())))
        }
    )
}
