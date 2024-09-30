//! Types related to task management & Functions for completely changing TCB

use super::id::TaskUserRes;
use super::stride::Stride;
use super::{kstack_alloc, KernelStack, ProcessControlBlock, TaskContext};
use crate::config::MAX_SYSCALL_NUM;
use crate::trap::TrapContext;
use crate::{mem::PhysPageNum, sync::UPSafeCell};
use alloc::sync::{Arc, Weak};
use core::cell::RefMut;

/// Task control block structure
pub struct TaskControlBlock {
    /// immutable
    pub process: Weak<ProcessControlBlock>,
    /// Kernel stack corresponding to PID
    pub kstack: KernelStack,
    /// mutable
    inner: UPSafeCell<TaskControlBlockInner>,
    /// Priority stride
    pub stride: Stride,
}

impl TaskControlBlock {
    /// Get the mutable reference of the inner TCB
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    /// Get the address of app's page table
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner_exclusive_access();
        inner.memory_set.token()
    }
    /// Set priority
    pub fn set_priority(self: &mut Arc<Self>, p: usize) -> usize {
        unsafe { Arc::get_mut_unchecked(self).stride.set_priority(p) }
        p
    }

    /// Accumulate stride
    pub fn accumulate_stride(self: &mut Arc<Self>) {
        unsafe {
            Arc::get_mut_unchecked(self).stride.accumulate();
        }
    }
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    /// The physical page number of the frame where the trap context is placed
    pub trap_cx_ppn: PhysPageNum,
    /// Save task context
    pub task_cx: TaskContext,

    /// Maintain the execution status of the current process
    pub task_status: TaskStatus,
    /// It is set when active exit or execution error occurs
    pub exit_code: Option<i32>,
    /// Total running time of process
    pub time: usize,
    /// The numbers of syscall called by process
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
}

impl TaskControlBlock {
    /// Create a new task
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        // println!("TaskControlBlock::new");
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        // println!("TaskControlBlock::new1");
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                    time: 0,
                    syscall_times: [0; MAX_SYSCALL_NUM]
                })
            },
            stride: Stride::new()
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// The execution status of the current process
pub enum TaskStatus {
    /// ready to run
    Ready,
    /// running
    Running,
    /// blocked
    Blocked,
}
