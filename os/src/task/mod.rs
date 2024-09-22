//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of `PID_ALLOCATOR` allocates pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod id;

mod manager;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::{config::MAX_SYSCALL_NUM, time::get_time_ms};

use crate::loader::{get_app_data_by_name, get_bin_data_by_name};
use alloc::sync::Arc;
use lazy_static::*;
pub use manager::fetch_task;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;
pub use id::{kstack_alloc, pid_alloc, KernelStack, PidHandle};
pub use manager::add_task;
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// We trace the syscalls with this method.
pub fn trace_syscall(syscall_id: usize) -> usize {
    // There must be an application running.
    let task = match current_task() {
        Some(t) => t,
        _ => {
            println!("THIS");
            return 1
        }
    };
    // ---- access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    inner.syscall_times[syscall_id % MAX_SYSCALL_NUM] += 1;
    0
}

use crate::syscall::TaskInfo;

/// Fetch task info
pub fn fetch_task_info() -> TaskInfo {
    // There must be an application running.
    let task = current_task().unwrap();
    // ---- access current TCB exclusively
    let inner = task.inner_exclusive_access();
    TaskInfo {
        time: get_time_ms() - inner.time,
        status: inner.task_status,
        syscall_times: inner.syscall_times
    }
}

/// mmap operation
pub fn current_task_memset_mmap(start: usize, len: usize, port: usize) -> isize {
    // There must be an application running.
    let task = current_task().unwrap();
    // ---- access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    let ms = &mut inner.memory_set;
    ms.mmap(start, len, port)
}

/// munmap operation
pub fn current_task_memset_munmap(start: usize, len: usize) -> isize {
    // There must be an application running.
    let task = current_task().unwrap();
    // ---- access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    let ms = &mut inner.memory_set;
    ms.munmap(start, len)
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        panic!("All applications completed!");
    }

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
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
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    /// Creation of initial process
    ///
    /// the name "initproc" may be changed to any other app name like "usertests",
    /// but we have user_shell, so we don't need to change it.
    pub static ref INITPROC_APP: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("shell_syscall").unwrap()
    ));
}

lazy_static! {
    /// Creation of initial process from binary
    pub static ref INITPROC_BINARY: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new(get_bin_data_by_name("shell_syscall").unwrap())
    );
}

#[allow(unused)]
/// Add init process to the manager
pub fn add_initproc_app() {
    add_task(INITPROC_APP.clone());
}

/// Add init process from binary to the manager
pub fn add_initproc_binary() {
    add_task(INITPROC_BINARY.clone());
}

#[allow(unused)]
/// Add user app to the manager
pub fn add_user_app(app_name: &str) {
    let app_data = get_app_data_by_name(app_name).unwrap();
    let task = TaskControlBlock::new(app_data);
    add_task(Arc::new(task));
}

/// Add user binary to the manager
pub fn add_user_binary(bin_name: &str) {
    let bin_data = get_bin_data_by_name(bin_name).unwrap();
    let task = TaskControlBlock::new(bin_data);
    add_task(Arc::new(task));
}
