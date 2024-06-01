//! Task context for scheduling
//!
//! The crate defines the needful fields for task context switching and scheduling.
//!
//! # Content
//!
//! - `tls`: Thread Local Storage (TLS) area for each task.
//!
//! - `stat`: Task statistics.
//!
//! - `preempt_disable_count`: Preemption disable counter. Only when the counter is zero, the
//! task can be preempted. It can be used to implement preemption protection lock.
#![no_std]
#![feature(naked_functions)]
#![feature(asm_const)]

mod arch;
mod current;
pub use current::*;

pub use arch::*;
#[cfg(feature = "tls")]
mod tls;

mod stat;
pub use stat::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "multitask")] {
        mod kstack;
        use kstack::*;mod task;
        pub use task::*;
    }
}

/// Disables kernel preemption.
///
/// It will increase the preemption disable counter of the current task.
#[cfg(feature = "preempt")]
pub fn disable_preempt() {
    let ptr: *const TaskInner = current_task_ptr();
    if !ptr.is_null() {
        unsafe {
            (*ptr).disable_preempt();
        }
    }
}

/// Enables kernel preemption.
///
/// It will decrease the preemption disable counter of the current task.Once the counter is zero, the
/// task can be preempted.
#[cfg(feature = "preempt")]
pub fn enable_preempt() {
    let ptr: *const TaskInner = current_task_ptr();
    if !ptr.is_null() {
        unsafe {
            (*ptr).enable_preempt();
        }
    }
}
