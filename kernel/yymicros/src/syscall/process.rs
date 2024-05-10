//! App management syscalls
use crate::process::run_next_app;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    run_next_app();
    panic!("process exited with code {}", exit_code);
}