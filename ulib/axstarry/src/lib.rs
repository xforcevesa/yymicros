//! The entry of syscall, which will distribute the syscall to the corresponding function
#![cfg_attr(all(not(test), not(doc)), no_std)]

extern crate alloc;
extern crate arch_boot;
mod file;
pub use file::fs_init;
mod api;
pub use api::{println, run_testcase};
#[cfg(feature = "ext4fs")]
#[allow(unused_imports)]
use axlibc::ax_open;

pub use linux_syscall_api::{init_current_dir, recycle_user_process};
