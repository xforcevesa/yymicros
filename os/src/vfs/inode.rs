use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::mem::UserBuffer;

#[allow(dead_code)]
/// trait File for all file types
pub trait File: Send + Sync {
    /// the file readable?
    fn readable(&self) -> bool;
    /// the file writable?
    fn writable(&self) -> bool;
    /// read from the file to buf, return the number of bytes read
    fn read(&self, buf: UserBuffer) -> usize;
    /// write to the file from buf, return the number of bytes written
    fn write(&self, buf: UserBuffer) -> usize;
    /// stat of file
    fn stat(&self) -> Option<Stat>;
}


/// The stat of a inode
#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    /// ID of device containing file
    pub dev: u64,
    /// inode number
    pub ino: u64,
    /// file type and mode
    pub mode: StatMode,
    /// number of hard links
    pub nlink: u32,
    /// unused pad
    pad: [u64; 7],
}

bitflags! {
    /// The mode of a inode
    /// whether a directory or a file
    pub struct StatMode: u32 {
        /// null
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}

use crate::sbi::{console_getchar, console_putchar};
use crate::sync::UPSafeCell;
use crate::process::suspend_current_and_run_next;
use crate::vfs::fs::ROOT_DIR;

use super::VfsNodeRef;

/// stdin file for getting chars from console
pub struct Stdin;

/// stdout file for putting chars to console
pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, mut user_buf: UserBuffer) -> usize {
        // Read multiple chars from console
        let mut read_size = 0;
        for slice in user_buf.buffers.iter_mut() {
            let mut i = 0;
            while i < slice.len() {
                let c = console_getchar();
                // echo
                console_putchar(c);
                if c == 0 {
                    suspend_current_and_run_next();
                    // EOF
                    break;
                } else if c == '\n' as usize || c == '\r' as usize {
                    // newline
                    break;
                } else if c == '\x7f' as usize {
                    if i > 0 {
                        i -= 1;
                        slice[i] = 0;
                    }
                    // delete char on left
                    console_putchar('\x08' as usize);
                    console_putchar(' ' as usize);
                    console_putchar('\x08' as usize);
                    continue;
                }
                slice[i] = c as u8;
                i += 1;
            }
            read_size += i;
            // Print the slice as string
            // println!("{}, read_size: {}", core::str::from_utf8(&slice[..i]).unwrap(), read_size);
        }
        read_size
    }
    fn write(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
    fn stat(&self) -> Option<Stat> {
        None
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
    fn stat(&self) -> Option<Stat> {
        None
    }
}

#[allow(dead_code)]
/// inode in memory
/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>,
}

#[allow(dead_code)]
/// The OS inode inner in 'UPSafeCell'
 pub struct OSInodeInner {
    offset: usize,
    inode: Arc<VfsNodeRef>,
}

#[allow(dead_code)]
impl OSInode {
    /// create a new inode in memory
    pub fn new(readable: bool, writable: bool, inode: Arc<VfsNodeRef>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPSafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    /// read all data from the inode
    pub fn read_all(&self) -> Vec<u8> {
        let inner = self.inner.exclusive_access();
        let mut buffer = [0u8; 512];
        let mut offset = 0;
        let mut v: Vec<u8> = Vec::new();
        while let Ok(size) = inner.inode.read_at(offset, &mut buffer) {
            offset += size as u64;
            v.extend_from_slice(&buffer[..size]);
            if size < buffer.len() {
                break;
            }
        }
        v
    }

    /// check if the inode is flag deleted
    pub fn is_deleted(&self, _name: &str) -> bool {
        // self.inner.exclusive_access().inode.is_removed(name)
        false
    }

    /// check if the inode is a link
    pub fn is_link(&self) -> bool {
        // self.inner.exclusive_access().inode.is_link()
        false
    }
}

bitflags! {
    ///  The flags argument to the open() system call is constructed by ORing together zero or more of the following values:
    pub struct OpenFlags: u32 {
        /// readyonly
        const RDONLY = 0;
        /// writeonly
        const WRONLY = 1 << 0;
        /// read and write
        const RDWR = 1 << 1;
        /// create new file
        const CREATE = 1 << 9;
        /// truncate file size to 0
        const TRUNC = 1 << 10;
    }
}

#[allow(dead_code)]
impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

lazy_static! {
    static ref ROOT_INODE: Arc<VfsNodeRef> = Arc::new(ROOT_DIR.as_ref().main_fs.root_dir());
}

#[allow(unused)]
/// Open a file
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Ok(inode) = ROOT_INODE.lookup(name) {
            // clear size
            inode.clear().unwrap();
            Some(Arc::new(OSInode::new(readable, writable, inode.into())))
        } else {
            // create file
            ROOT_INODE
                .create(name, super::VfsNodeType::File)
                .map_or(None, |inode| Some(Arc::new(OSInode::new(readable, writable, inode.into()))))
        }
    } else {
        let ll = ROOT_INODE.lookup(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear().unwrap();
            }
            Arc::new(OSInode::new(readable, writable, inode.clone().into()))
        });
        match ll {
            Ok(inode) => Some(inode),
            _ => None,
        }
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset as u64, *slice).unwrap_or(0);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset as u64, *slice).unwrap_or(0);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
    fn stat(&self) -> Option<Stat> {
        let inner = self.inner.exclusive_access();

        Some(Stat {
            dev: 0,
            ino: inner.inode.ino().unwrap().into(),
            mode: {
                match inner.inode.is_dir() {
                    Ok(true) => StatMode::DIR,
                    Ok(false) => StatMode::FILE,
                    Err(_) => StatMode::NULL,
                }
            },
            nlink: {
                let map = INODE_LINK_MAP.exclusive_access();
                let inner_inode = inner.inode.ino();
                let count = map
                    .get(&inner_inode.unwrap().into())
                    .cloned()
                    .unwrap_or(1);

                count as u32
            },
            pad: [0; 7],
        })
    }
}

lazy_static! {
    pub static ref INODE_LINK_MAP: UPSafeCell<BTreeMap<u64, u64>> = {
        let map = BTreeMap::new();
        unsafe { UPSafeCell::new(map) }
    };
}

#[allow(unused)]
/// link two files
pub fn link_file(old_name: &str, new_name: &str) -> isize {
    if old_name == new_name {
        return -1;
    }

    if let Ok(old_inode) = ROOT_INODE.lookup(old_name) {
        if let Ok(new_inode) = ROOT_INODE.lookup(new_name) {
            // increments link count
            let mut inner = INODE_LINK_MAP.exclusive_access();
            let key = new_inode.ino().unwrap();
            let old_inode = old_inode.ino().unwrap();
            let old_count = inner.get(&key).cloned().unwrap_or(1);
            inner.insert(old_inode, old_count + 1);
            0
        } else {
            -1
        }
    } else {
        -1
    }
}

#[allow(unused)]
/// unlink a file
pub fn unlink_file(file_name: &str) -> isize {
    if let Ok(inode) = ROOT_INODE.lookup(file_name) {
        // flag in remove
        inode.unlink(file_name);

        // decrease link count
        let inode_num = inode.ino().unwrap();
        let mut inner = INODE_LINK_MAP.exclusive_access();
        let old_count = inner.get(&inode_num).cloned().unwrap_or(1);
        inner.insert(inode_num, old_count - 1);

        if old_count == 0 {
            inner.remove(&inode_num);
        }

        0
    } else {
        -1
    }
}

