//! File and filesystem-related syscalls
use crate::vfs::{inode::{link_file, open_file, unlink_file}, inode::{OpenFlags, Stat}};
use crate::mem::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::process::{current_process, current_user_token};

/// Performs a syscall that writes a file.
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_process().unwrap().pid.0);
    let token = current_user_token();
    let process = current_process().unwrap();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current process TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

/// Performs a syscall that reads a file.
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_process().unwrap().pid.0);
    let token = current_user_token();
    let process = current_process().unwrap();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current process TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

/// Performs a syscall that opens a file.
pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_process().unwrap().pid.0);
    let process = current_process().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    let flags = OpenFlags::from_bits(flags).unwrap();
    if let Some(inode) = open_file(path.as_str(), flags) {
        if !flags.contains(OpenFlags::CREATE) && inode.is_deleted(path.as_str()) {
            return -1;
        }

        let mut inner = process.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

/// Performs a syscall that closes a file descriptor.
pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_process().unwrap().pid.0);
    let process = current_process().unwrap();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat",
        current_process().unwrap().pid.0
    );

    let process = current_process().unwrap();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release Process lock manually to avoid deadlock
        drop(inner);
        if let Some(stat) = file.stat() {
            let token = current_user_token();

            *translated_refmut(token, st) = stat;

            return 0;
        }
        -1
    } else {
        -1
    }
}

/// YOUR JOB: Implement link_file.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat",
        current_process().unwrap().pid.0
    );

    let token = current_user_token();

    let old_name = translated_str(token, old_name);
    let new_name = translated_str(token, new_name);

    link_file(
        old_name.as_str(),
        new_name.as_str(),
    )
}

/// YOUR JOB: Implement unlink_file.
pub fn sys_unlinkat(name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat",
        current_process().unwrap().pid.0
    );

    let token = current_user_token();

    let name = translated_str(token, name);

    unlink_file(name.as_str())
}
