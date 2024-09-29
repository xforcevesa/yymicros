//! Conditian variable

use crate::sync::{Mutex, UPSafeCell};
use crate::process::{block_current_and_run_next, current_process, wakeup_process, ProcessControlBlock};
use alloc::{collections::VecDeque, sync::Arc};

/// Condition variable structure
pub struct Condvar {
    /// Condition variable inner
    pub inner: UPSafeCell<CondvarInner>,
}

pub struct CondvarInner {
    pub wait_queue: VecDeque<Arc<ProcessControlBlock>>,
}

impl Condvar {
    /// Create a new condition variable
    pub fn new() -> Self {
        trace!("kernel: Condvar::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(CondvarInner {
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    /// Signal a process waiting on the condition variable
    pub fn signal(&self) {
        let mut inner = self.inner.exclusive_access();
        if let Some(process) = inner.wait_queue.pop_front() {
            wakeup_process(process);
        }
    }

    /// blocking current process, let it wait on the condition variable
    pub fn wait(&self, mutex: Arc<dyn Mutex>) {
        trace!("kernel: Condvar::wait_with_mutex");
        mutex.unlock();
        let mut inner = self.inner.exclusive_access();
        inner.wait_queue.push_back(current_process().unwrap());
        drop(inner);
        block_current_and_run_next();
        mutex.lock();
    }
}
