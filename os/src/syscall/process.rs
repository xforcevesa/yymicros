//! Process management syscalls
use core::borrow::BorrowMut;

use alloc::sync::Arc;

use crate::{
    config::MAX_SYSCALL_NUM,
    loader::get_app_data_by_name,
    mem::{translated_refmut, translated_str},
    process::{
        add_process, current_process, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, ProcessStatus, current_process_memset_mmap,
        current_process_memset_munmap, fetch_process_info
    }, time::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Process information
#[allow(dead_code)]
pub struct ProcessInfo {
    /// Process status in it's life cycle
    pub status: ProcessStatus,
    /// The numbers of syscall called by process
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of process
    pub time: usize,
}

/// process exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_process().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current process gives up resources for other processes
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_process().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_process().unwrap().pid.0);
    current_process().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_process().unwrap().pid.0);
    let current_process = current_process().unwrap();
    let new_process = current_process.fork();
    let new_pid = new_process.pid.0;
    // modify trap context of new_process, because it returns immediately after switching
    let trap_cx = new_process.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new process to scheduler
    add_process(new_process);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_process().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let process = current_process().unwrap();
        process.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_process().unwrap().pid.0, pid);
    let process = current_process().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = process.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time",
        current_process().unwrap().pid.0
    );
    let us = get_time_us();
    *translated_refmut(current_user_token(), ts) = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    0
}

/// YOUR JOB: Finish sys_process_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`ProcessInfo`] is splitted by two pages ?
pub fn sys_process_info(ti: *mut ProcessInfo) -> isize {
    trace!(
        "kernel:pid[{}] sys_process_info",
        current_process().unwrap().pid.0
    );
    *translated_refmut(current_user_token(), ti) = fetch_process_info();
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap",
        current_process().unwrap().pid.0
    );
    current_process_memset_mmap(start, len, port)
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap",
        current_process().unwrap().pid.0
    );
    current_process_memset_munmap(start, len)
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_process().unwrap().pid.0);
    if let Some(old_brk) = current_process().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn",
        current_process().unwrap().pid.0
    );
    let current_process = current_process().unwrap();
    let new_process = 
        current_process.spawn(&translated_str(current_user_token(), path));
    let new_process = match new_process {
        Some(process) => process,
        None => return -1
    };
    let new_pid = new_process.pid.0;
    // // modify trap context of new_process, because it returns immediately after switching
    // let trap_cx = new_process.inner_exclusive_access().get_trap_cx();
    // // we do not have to move to next instruction since we have done it before
    // // for child process, fork returns 0
    // trap_cx.x[10] = 0;
    // add new process to scheduler
    add_process(new_process);
    new_pid as isize
}

// YOUR JOB: Set process priority.
pub fn sys_set_priority(prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_process().unwrap().pid.0
    );
    if prio <= 1 {
        -1
    } else {
        let mut current_process = current_process().unwrap();
        let current_process = current_process.borrow_mut();
        current_process.set_priority(prio as usize) as isize
    }
}
