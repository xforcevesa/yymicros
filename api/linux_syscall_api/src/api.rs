extern crate alloc;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use axhal::{
    arch::{flush_tlb, write_page_table_root},
    KERNEL_PROCESS_ID,
};
use axprocess::{wait_pid, yield_now_task, Process, PID2PC};
use axruntime::KERNEL_PAGE_TABLE;
use axtask::{TaskId, EXITED_TASKS};

use axfs::api::OpenFlags;

/// 在完成一次系统调用之后，恢复全局目录
pub fn init_current_dir() {
    axfs::api::set_current_dir("/").expect("reset current dir failed");
}

/// Flags for opening a file
pub type FileFlags = OpenFlags;

/// 释放所有非内核进程
pub fn recycle_user_process() {
    let kernel_process = Arc::clone(PID2PC.lock().get(&KERNEL_PROCESS_ID).unwrap());

    loop {
        let pid2pc = PID2PC.lock();

        kernel_process
            .children
            .lock()
            .retain(|x| x.pid() == KERNEL_PROCESS_ID || pid2pc.contains_key(&x.pid()));
        let all_finished = pid2pc.len() == 1;
        drop(pid2pc);
        if all_finished {
            break;
        }
        yield_now_task();
    }
    TaskId::clear();
    unsafe {
        write_page_table_root(KERNEL_PAGE_TABLE.root_paddr());
        flush_tlb(None);
    };
    EXITED_TASKS.lock().clear();
    init_current_dir();
}

/// To read a file with the given path
pub fn read_file(path: &str) -> Option<String> {
    axfs::api::read_to_string(path).ok()
}

/// To run a testcase with the given name and environment variables, which will be used in initproc
pub fn run_testcase(testcase: &str, envs: Vec<String>) {
    axlog::ax_println!("Running testcase: {}", testcase);
    let args = testcase.split_whitespace();
    let mut args_vec: Vec<String> = Vec::new();
    for arg in args {
        args_vec.push(arg.to_string());
    }

    let user_process = Process::init(args_vec, &envs).unwrap();
    let now_process_id = user_process.get_process_id() as isize;
    let mut exit_code = 0;
    loop {
        if unsafe { wait_pid(now_process_id, &mut exit_code as *mut i32) }.is_ok() {
            break;
        }
        yield_now_task();
    }
    recycle_user_process();
    axlog::ax_println!(
        "Testcase {} finished with exit code {}",
        testcase,
        exit_code
    );
}
