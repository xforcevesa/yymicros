//!Wrap `switch.S` as a function
use super::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

extern "C" {
    /// Switch to the context of `next_process_cx_ptr`, saving the current context
    /// in `current_process_cx_ptr`.
    pub fn __switch(current_process_cx_ptr: *mut TaskContext, next_process_cx_ptr: *const TaskContext);
}
