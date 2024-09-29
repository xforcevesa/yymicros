//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::{fetch_process, ProcessStatus};
use super::{TaskContext, ProcessControlBlock};
use crate::sync::UPSafeCell;
use crate::time::get_time_ms;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The process currently executing on the current processor
    current: Option<Arc<ProcessControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_process_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_process_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_process_cx`
    fn get_idle_process_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_process_cx as *mut _
    }

    ///Get current process in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.current.take()
    }

    ///Get current process in cloning semanteme
    pub fn current(&self) -> Option<Arc<ProcessControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_process` to get the process that needs to run, and switch the process through `__switch`
pub fn run_processes() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(process) = fetch_process() {
            let idle_process_cx_ptr = processor.get_idle_process_cx_ptr();
            // access coming process TCB exclusively
            let mut process_inner = process.inner_exclusive_access();
            let next_process_cx_ptr = &process_inner.process_cx as *const TaskContext;
            process_inner.process_status = ProcessStatus::Running;
            if process_inner.time == 0 {
                process_inner.time = get_time_ms()
            }
            // release coming process_inner manually
            drop(process_inner);
            // release coming process TCB manually
            processor.current = Some(process);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_process_cx_ptr, next_process_cx_ptr);
            }
        } else {
            warn!("no processes available in run_processes");
        }
    }
}

/// Get current process through take, leaving a None in its place
pub fn take_current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current process
pub fn current_process() -> Option<Arc<ProcessControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let process = current_process().unwrap();
    process.get_user_token()
}

///Get the mutable reference to trap context of current process
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_process()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_process_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_process_cx_ptr = processor.get_idle_process_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_process_cx_ptr, idle_process_cx_ptr);
    }
}
