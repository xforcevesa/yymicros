//! Implementation of [`TaskContext`]
use crate::trap::trap_return;

#[repr(C)]
#[derive(Debug)]
/// task context structure containing some registers
pub struct TaskContext {
    /// Ret position after task switching
    ra: usize,
    /// Stack pointer
    pub sp: usize,
    /// s0-11 register, callee saved
    s: [usize; 12],
    #[allow(unused)]
    /// ptr for green thread
    pub thread_ptr: usize
}

impl TaskContext {
    /// Create a new empty task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
            thread_ptr: 0
        }
    }
    /// Create a new task context with a trap return addr and a kernel stack pointer
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
            thread_ptr: 0
        }
    }
}
