use alloc::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use spinlock::SpinNoIrq;

use crate::{AxRunQueue, AxTaskRef, WaitQueue};

/// A map to store tasks' wait queues, which stores tasks that are waiting for this task to exit.
pub(crate) static WAIT_FOR_TASK_EXITS: SpinNoIrq<BTreeMap<u64, Arc<WaitQueue>>> =
    SpinNoIrq::new(BTreeMap::new());

#[cfg(feature = "irq")]
/// A list to store tasks that need to be woken up by timer.
static TASK_IN_TIME_LIST: SpinNoIrq<BTreeSet<u64>> = SpinNoIrq::new(BTreeSet::new());

/// A queue to store sleeping tasks.
static TASK_IN_WAIT_QUEUE: SpinNoIrq<BTreeSet<u64>> = SpinNoIrq::new(BTreeSet::new());

#[cfg(feature = "irq")]
pub(crate) fn in_timer_list(task: &AxTaskRef) -> bool {
    TASK_IN_TIME_LIST.lock().contains(&task.id().as_u64())
}

pub(crate) fn in_wait_queue(task: &AxTaskRef) -> bool {
    TASK_IN_WAIT_QUEUE.lock().contains(&task.id().as_u64())
}

#[cfg(feature = "irq")]
pub(crate) fn add_to_timer_list(task: &AxTaskRef) {
    TASK_IN_TIME_LIST.lock().insert(task.id().as_u64());
}

pub(crate) fn add_to_wait_queue(task: &AxTaskRef) {
    TASK_IN_WAIT_QUEUE.lock().insert(task.id().as_u64());
}

#[cfg(feature = "irq")]
pub(crate) fn remove_from_timer_list(task: &AxTaskRef) {
    TASK_IN_TIME_LIST.lock().remove(&task.id().as_u64());
}

pub(crate) fn remove_from_wait_queue(task: &AxTaskRef) {
    TASK_IN_WAIT_QUEUE.lock().remove(&task.id().as_u64());
}

pub(crate) fn add_wait_for_exit_queue(task: &AxTaskRef) {
    WAIT_FOR_TASK_EXITS
        .lock()
        .insert(task.id().as_u64(), Arc::new(WaitQueue::new()));
}

pub(crate) fn get_wait_for_exit_queue(task: &AxTaskRef) -> Option<Arc<WaitQueue>> {
    WAIT_FOR_TASK_EXITS.lock().get(&task.id().as_u64()).cloned()
}

/// When the task exits, notify all tasks that are waiting for this task to exit, and
/// then remove the wait queue of the exited task.
pub(crate) fn notify_wait_for_exit(task: &AxTaskRef, rq: &mut AxRunQueue) {
    if let Some(wait_queue) = WAIT_FOR_TASK_EXITS.lock().remove(&task.id().as_u64()) {
        wait_queue.notify_all_locked(true, rq);
    }
}
