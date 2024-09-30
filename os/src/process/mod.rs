//! Process management implementation
//!
//! Everything about process management, like starting and switching processes is
//! implemented here.
//!
//! A single global instance of [`ProcessManager`] called `TASK_MANAGER` controls
//! all the processes in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! process(s) for each core.
//!
//! A single global instance of `PID_ALLOCATOR` allocates pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod id;
mod stride;
mod manager;
mod processor;
mod switch;
#[allow(dead_code)]
mod thread;
#[allow(clippy::module_inception)]
mod process;

use crate::{config::MAX_SYSCALL_NUM, time::get_time_ms};

use crate::loader::{get_app_data_by_name, get_bin_data_by_name};
use alloc::sync::Arc;
use lazy_static::*;
pub use manager::fetch_process;
use switch::__switch;
pub use process::{ProcessControlBlock, ProcessStatus};

pub use context::TaskContext;
pub use id::{kstack_alloc, pid_alloc, KernelStack, PidHandle};
pub use manager::{add_process, wakeup_process};
pub use processor::{
    current_process, current_trap_cx, current_user_token, run_processes, schedule, take_current_process};
/// Suspend the current 'Running' process and run the next process in process list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let process = take_current_process().unwrap();

    // ---- access current TCB exclusively
    let mut process_inner = process.inner_exclusive_access();
    let process_cx_ptr = &mut process_inner.process_cx as *mut TaskContext;
    // Change status to Ready
    process_inner.process_status = ProcessStatus::Ready;
    drop(process_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_process(process);
    // jump to scheduling cycle
    schedule(process_cx_ptr);
}

/// We trace the syscalls with this method.
pub fn trace_syscall(syscall_id: usize) -> usize {
    // There must be an application running.
    let process = match current_process() {
        Some(t) => t,
        _ => {
            println!("THIS");
            return 1
        }
    };
    // ---- access current TCB exclusively
    let mut inner = process.inner_exclusive_access();
    inner.syscall_times[syscall_id % MAX_SYSCALL_NUM] += 1;
    0
}

use crate::syscall::ProcessInfo;

/// Fetch process info
pub fn fetch_process_info() -> ProcessInfo {
    // There must be an application running.
    let process = current_process().unwrap();
    // ---- access current TCB exclusively
    let inner = process.inner_exclusive_access();
    ProcessInfo {
        time: get_time_ms() - inner.time,
        status: inner.process_status,
        syscall_times: inner.syscall_times
    }
}

/// mmap operation
pub fn current_process_memset_mmap(start: usize, len: usize, port: usize) -> isize {
    // There must be an application running.
    let process = current_process().unwrap();
    // ---- access current TCB exclusively
    let mut inner = process.inner_exclusive_access();
    let ms = &mut inner.memory_set;
    ms.mmap(start, len, port)
}

/// munmap operation
pub fn current_process_memset_munmap(start: usize, len: usize) -> isize {
    // There must be an application running.
    let process = current_process().unwrap();
    // ---- access current TCB exclusively
    let mut inner = process.inner_exclusive_access();
    let ms = &mut inner.memory_set;
    ms.munmap(start, len)
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' process and run the next process in process list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let process = take_current_process().unwrap();

    let pid = process.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        panic!("All applications completed!");
    }

    // **** access current TCB exclusively
    let mut inner = process.inner_exclusive_access();
    // Change status to Zombie
    inner.process_status = ProcessStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC_APP.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC_APP));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB
    // drop process manually to maintain rc correctly
    drop(process);
    // we do not have to save process context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

/// Make current process blocked and switch to the next process.
pub fn block_current_and_run_next() {
    let process = take_current_process().unwrap();
    let mut process_inner = process.inner_exclusive_access();
    let process_cx_ptr = &mut process_inner.process_cx as *mut TaskContext;
    process_inner.process_status = ProcessStatus::Blocked;
    drop(process_inner);
    schedule(process_cx_ptr);
}

lazy_static! {
    /// Creation of initial process
    ///
    /// the name "initproc" may be changed to any other app name like "usertests",
    /// but we have user_shell, so we don't need to change it.
    pub static ref INITPROC_APP: Arc<ProcessControlBlock> = Arc::new(ProcessControlBlock::new(
        get_app_data_by_name("shell_syscall").unwrap()
    ));
}

lazy_static! {
    /// Creation of initial process from binary
    pub static ref INITPROC_BINARY: Arc<ProcessControlBlock> = Arc::new(
        ProcessControlBlock::new(get_bin_data_by_name("shell_syscall").unwrap())
    );
}

#[allow(unused)]
/// Add init process to the manager
pub fn add_initproc_app() {
    add_process(INITPROC_APP.clone());
}

/// Add init process from binary to the manager
pub fn add_initproc_binary() {
    add_process(INITPROC_BINARY.clone());
}

#[allow(unused)]
/// Add user app to the manager
pub fn add_user_app(app_name: &str) {
    let app_data = get_app_data_by_name(app_name).unwrap();
    let process = ProcessControlBlock::new(app_data);
    add_process(Arc::new(process));
}

/// Add user binary to the manager
pub fn add_user_binary(bin_name: &str) {
    let bin_data = get_bin_data_by_name(bin_name).unwrap();
    let process = ProcessControlBlock::new(bin_data);
    add_process(Arc::new(process));
}
