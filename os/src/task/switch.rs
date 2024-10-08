//!provides __switch_task asm function to switch between two task contexts  [`TaskContext`]
use super::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

extern "C" {
    /// Switch to the context of `next_task_cx_ptr`, saving the current context
    /// in `current_task_cx_ptr`.
    pub fn __switch_task(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
