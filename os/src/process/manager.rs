//!Implementation of [`ProcessManager`]
use super::ProcessControlBlock;
use crate::sync::UPSafeCell;
use crate::process::ProcessStatus;
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use lazy_static::*;
use core::cmp::Ordering;

///A array of `ProcessControlBlock` that is thread-safe
pub struct ProcessManager {
    // ready_queue: VecDeque<Arc<ProcessControlBlock>>,
    heap: BinaryHeap<Arc<ProcessControlBlock>>
}

impl PartialOrd for ProcessControlBlock {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.stride, &other.stride)
    }
}

impl PartialEq for ProcessControlBlock {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Ord for ProcessControlBlock {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match PartialOrd::partial_cmp(&self.stride, &other.stride) {
            Some(t) => t,
            None => Ordering::Equal
        }
    }
}

impl Eq for ProcessControlBlock {
    
}

/// A simple FIFO scheduler.
impl ProcessManager {
    ///Creat an empty ProcessManager
    pub fn new() -> Self {
        Self {
            // ready_queue: VecDeque::new(),
            heap: BinaryHeap::new()
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, mut process: Arc<ProcessControlBlock>) {
        // Potential BUG here.
        process.accumulate_stride();
        self.heap.push(process);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.heap.pop()
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<ProcessManager> =
        unsafe { UPSafeCell::new(ProcessManager::new()) };
}

/// Add process to ready queue
pub fn add_process(process: Arc<ProcessControlBlock>) {
    //trace!("kernel: ProcessManager::add_process");
    TASK_MANAGER.exclusive_access().add(process);
}

/// Take a process out of the ready queue
pub fn fetch_process() -> Option<Arc<ProcessControlBlock>> {
    //trace!("kernel: ProcessManager::fetch_process");
    TASK_MANAGER.exclusive_access().fetch()
}

/// Wake up a process
pub fn wakeup_process(process: Arc<ProcessControlBlock>) {
    trace!("kernel: ProcessManager::wakeup_process");
    let mut process_inner = process.inner_exclusive_access();
    process_inner.process_status = ProcessStatus::Ready;
    drop(process_inner);
    add_process(process);
}
