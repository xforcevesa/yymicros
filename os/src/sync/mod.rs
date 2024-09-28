//! Uniprocessor interior mutability primitives
mod lazy_init;

#[allow(dead_code)]
mod condvar;
#[allow(dead_code)]
mod mutex;
#[allow(dead_code)]
mod semaphore;

pub use lazy_init::{LazyInit, UPSafeCell};
#[allow(unused)]
pub use condvar::Condvar;
pub use mutex::Mutex;
#[allow(unused)]
pub use semaphore::Semaphore;
