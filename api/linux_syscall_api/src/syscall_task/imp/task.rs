use core::mem::size_of;
/// 处理与任务（线程）有关的系统调用
use core::time::Duration;

use axconfig::TASK_STACK_SIZE;
use axhal::time::current_time;
use axprocess::{
    current_process, current_task, exit_current_task,
    flags::{CloneFlags, WaitStatus},
    futex::clear_wait,
    link::{deal_with_path, raw_ptr_to_ref_str, AT_FDCWD},
    set_child_tid, sleep_now_task, wait_pid, yield_now_task, Process, PID2PC,
};
use axsync::Mutex;
// use axtask::{
//     monolithic_task::task::{SchedPolicy, SchedStatus},
//     AxTaskRef,
// };
use crate::{
    CloneArgs, RLimit, SyscallError, SyscallResult, TimeSecs, WaitFlags, RLIMIT_AS, RLIMIT_NOFILE,
    RLIMIT_STACK,
};
use axlog::{info, warn};
use axtask::TaskId;
extern crate alloc;

use alloc::{string::ToString, sync::Arc, vec::Vec};

use axsignal::signal_no::SignalNo;

use axprocess::signal::SignalModule;
// pub static TEST_FILTER: Mutex<BTreeMap<String, usize>> = Mutex::new(BTreeMap::new());

/// # Arguments
/// * `exit_code` - i32
pub fn syscall_exit(args: [usize; 6]) -> ! {
    let exit_code = args[0] as i32;
    info!("exit: exit_code = {}", exit_code);
    // let cases = ["fcanf", "fgetwc_buffering", "lat_pipe"];
    // let mut test_filter = TEST_FILTER.lock();
    // for case in cases {
    //     let case = case.to_string();
    //     if test_filter.contains_key(&case) {
    //         test_filter.remove(&case);
    //     }
    // }
    // drop(test_filter);
    exit_current_task(exit_code)
}

// /// 过滤掉不想测的测例，比赛时使用
// ///
// /// 若不想测该测例，返回false
// pub fn filter(testcase: String) -> bool {
//     let mut test_filter = TEST_FILTER.lock();
//     if testcase == "./fstime".to_string()
//         || testcase == "fstime".to_string()
//         || testcase == "looper".to_string()
//         || testcase == "./looper".to_string()
//     {
//         if test_filter.contains_key(&testcase) {
//             let count = test_filter.get_mut(&testcase).unwrap();
//             if (testcase == "./fstime".to_string() || testcase == "fstime".to_string())
//                 && *count == 6
//             {
//                 return false;
//             }
//             *count += 1;
//         } else {
//             if testcase == "looper".to_string() || testcase == "./looper".to_string() {
//                 return false;
//             }
//             test_filter.insert(testcase, 1);
//         }
//     } else {
//         // 记录有无即可

//         test_filter.insert(testcase, 1);
//     }
//     true
// }

/// # Arguments
/// * `path` - *const u8
/// * `argv` - *const usize
/// * `envp` - *const usize
pub fn syscall_exec(args: [usize; 6]) -> SyscallResult {
    let path = args[0] as *const u8;
    let mut argv = args[1] as *const usize;
    let mut envp = args[2] as *const usize;
    let path = deal_with_path(AT_FDCWD, Some(path), false);
    if path.is_none() {
        return Err(SyscallError::EINVAL);
    }
    let path = path.unwrap();
    if path.is_dir() {
        return Err(SyscallError::EISDIR);
    }
    let path = path.path().to_string();

    let mut args_vec = Vec::new();
    // args相当于argv，指向了参数所在的地址
    loop {
        let args_str_ptr = unsafe { *argv };
        if args_str_ptr == 0 {
            break;
        }
        args_vec.push(unsafe { raw_ptr_to_ref_str(args_str_ptr as *const u8) }.to_string());
        unsafe {
            argv = argv.add(1);
        }
    }
    let mut envs_vec = Vec::new();
    if envp as usize != 0 {
        loop {
            let envp_str_ptr = unsafe { *envp };
            if envp_str_ptr == 0 {
                break;
            }
            envs_vec.push(unsafe { raw_ptr_to_ref_str(envp_str_ptr as *const u8) }.to_string());
            unsafe {
                envp = envp.add(1);
            }
        }
    }
    // let testcase = if args_vec[0] == "./busybox".to_string()
    //     || args_vec[0] == "busybox".to_string()
    //     || args_vec[0] == "entry-static.exe".to_string()
    //     || args_vec[0] == "entry-dynamic.exe".to_string()
    //     || args_vec[0] == "lmbench_all".to_string()
    // {
    //     args_vec[1].clone()
    // } else {
    //     args_vec[0].clone()
    // };
    // if filter(testcase) == false {
    //     return -1;
    // }
    let curr_process = current_process();

    // 设置 file_path
    curr_process.set_file_path(path.clone());

    // 清空futex信号列表
    clear_wait(curr_process.pid(), true);
    let argc = args_vec.len();
    if curr_process.exec(path, args_vec, &envs_vec).is_err() {
        exit_current_task(0);
    }
    Ok(argc as isize)
}

/// # Arguments for riscv
/// * `flags` - usize
/// * `user_stack` - usize
/// * `ptid` - usize
/// * `tls` - usize
/// * `ctid` - usize
///
/// # Arguments for x86_64
/// * `flags` - usize
/// * `user_stack` - usize
/// * `ptid` - usize
/// * `ctid` - usize
/// * `tls` - usize
pub fn syscall_clone(args: [usize; 6]) -> SyscallResult {
    let flags = args[0];
    let user_stack = args[1];
    let ptid = args[2];
    let tls: usize;
    let ctid: usize;
    #[cfg(target_arch = "x86_64")]
    {
        ctid = args[3];
        tls = args[4];
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        tls = args[3];
        ctid = args[4];
    }
    let clone_flags = CloneFlags::from_bits((flags & !0x3f) as u32).unwrap();

    let stack = if user_stack == 0 {
        None
    } else {
        Some(user_stack)
    };
    let curr_process = current_process();

    let sig_child = SignalNo::from(flags & 0x3f) == SignalNo::SIGCHLD;

    if let Ok(new_task_id) = curr_process.clone_task(clone_flags, stack, ptid, tls, ctid, sig_child)
    {
        Ok(new_task_id as isize)
    } else {
        Err(SyscallError::ENOMEM)
    }
}

/// 创建子进程的新函数，所有信息保存在 CloneArgs
/// # Arguments
/// * `clone_args` - *const CloneArgs
/// * `size` - usize
pub fn syscall_clone3(args: [usize; 6]) -> SyscallResult {
    let size = args[1];
    let clone_args = args[0] as *const CloneArgs;
    assert!(size >= size_of::<CloneArgs>());

    let curr_process = current_process();

    let args = match curr_process.manual_alloc_type_for_lazy(clone_args) {
        Ok(_) => unsafe { &*clone_args },
        Err(_) => return Err(SyscallError::EFAULT),
    };

    let clone_flags = CloneFlags::from_bits(args.flags as u32).unwrap();

    let stack = if args.stack == 0 {
        None
    } else {
        Some(args.stack as usize)
    };

    let sig_child = SignalNo::from(args.exit_signal as usize & 0x3f) == SignalNo::SIGCHLD;

    warn!("stack size  {}", args.stack_size);
    if let Ok(new_task_id) = curr_process.clone_task(
        clone_flags,
        stack,
        args.parent_tid as usize,
        args.tls as usize,
        args.child_tid as usize,
        sig_child,
    ) {
        Ok(new_task_id as isize)
    } else {
        Err(SyscallError::ENOMEM)
    }
}

/// 创建一个子进程，挂起父进程，直到子进程exec或者exit，父进程才继续执行
#[cfg(target_arch = "x86_64")]
pub fn syscall_vfork() -> SyscallResult {
    let args: [usize; 6] = [0x4011, 0, 0, 0, 0, 0];
    // TODO: check the correctness
    syscall_clone(args).map(|new_task_id| {
        let task_ref = axprocess::get_task_ref(new_task_id as u64).unwrap();
        axtask::vfork_suspend(&task_ref);
        new_task_id
    })
}

/// 等待子进程完成任务，若子进程没有完成，则自身yield
/// 当前仅支持WNOHANG选项，即若未完成时则不予等待，直接返回0
/// # Arguments
/// * `pid` - isize
/// * `exit_code_ptr` - *mut i32
/// * `option` - WaitFlags
pub fn syscall_wait4(args: [usize; 6]) -> SyscallResult {
    let pid = args[0] as isize;
    let exit_code_ptr = args[1] as *mut i32;
    let option = WaitFlags::from_bits(args[2] as u32).unwrap();
    loop {
        let answer = unsafe { wait_pid(pid, exit_code_ptr) };
        match answer {
            Ok(pid) => {
                return Ok(pid as isize);
            }
            Err(status) => {
                match status {
                    WaitStatus::NotExist => {
                        return Err(SyscallError::EPERM);
                    }
                    WaitStatus::Running => {
                        if option.contains(WaitFlags::WNOHANG) {
                            // 不予等待，直接返回0
                            return Ok(0);
                        } else {
                            // wait回来之后，如果还需要wait，先检查是否有信号未处理

                            if current_process().have_signals().is_some() {
                                return Err(SyscallError::EINTR);
                            }
                            // 执行yield操作，切换任务
                            yield_now_task();
                        }
                    }
                    _ => {
                        panic!("Shouldn't reach here!");
                    }
                }
            }
        };
    }
}

/// To yield the current task
pub fn syscall_yield() -> SyscallResult {
    yield_now_task();
    Ok(0)
}

/// 当前任务进入睡眠，req指定了睡眠的时间
/// rem存储当睡眠完成时，真实睡眠时间和预期睡眠时间之间的差值
/// # Arguments
/// * `req` - *const TimeSecs
/// * `rem` - *mut TimeSecs
pub fn syscall_sleep(args: [usize; 6]) -> SyscallResult {
    let req = args[0] as *const TimeSecs;
    let rem = args[1] as *mut TimeSecs;
    let req_time = unsafe { *req };
    let start_to_sleep = current_time();
    // info!("sleep: req_time = {:?}", req_time);
    let dur = Duration::new(req_time.tv_sec as u64, req_time.tv_nsec as u32);
    sleep_now_task(dur);
    // 若被唤醒时时间小于请求时间，则将剩余时间写入rem
    let sleep_time = current_time() - start_to_sleep;
    if rem as usize != 0 {
        if sleep_time < dur {
            let delta = (dur - sleep_time).as_nanos() as usize;
            unsafe {
                *rem = TimeSecs {
                    tv_sec: delta / 1_000_000_000,
                    tv_nsec: delta % 1_000_000_000,
                }
            };
        } else {
            unsafe {
                *rem = TimeSecs {
                    tv_sec: 0,
                    tv_nsec: 0,
                }
            };
        }
    }

    if current_process().have_signals().is_some() {
        return Err(SyscallError::EINTR);
    }
    Ok(0)
}

/// 设置tid对应的指针
/// 返回值为当前的tid
/// # Arguments
/// * `tid` - usize
pub fn syscall_set_tid_address(args: [usize; 6]) -> SyscallResult {
    let tid = args[0];
    set_child_tid(tid);
    Ok(current_task().id().as_u64() as isize)
}

/// 设置任务资源限制
///
/// pid 设为0时，表示应用于自己
///
/// # Arguments
/// * `pid` - usize
/// * `resource` - i32
/// * `new_limit` - *const RLimit
/// * `old_limit` - *mut RLimit
pub fn syscall_prlimit64(args: [usize; 6]) -> SyscallResult {
    let pid = args[0];
    let resource = args[1] as i32;
    let new_limit = args[2] as *const RLimit;
    let old_limit = args[3] as *mut RLimit;
    // 当pid不为0，其实没有权利去修改其他的进程的资源限制
    let curr_process = current_process();
    if pid == 0 || pid == curr_process.pid() as usize {
        match resource {
            RLIMIT_STACK => {
                if old_limit as usize != 0 {
                    unsafe {
                        *old_limit = RLimit {
                            rlim_cur: TASK_STACK_SIZE as u64,
                            rlim_max: TASK_STACK_SIZE as u64,
                        };
                    }
                }
            }
            RLIMIT_NOFILE => {
                // 仅支持修改最大文件数
                if old_limit as usize != 0 {
                    let limit = curr_process.fd_manager.get_limit();
                    unsafe {
                        *old_limit = RLimit {
                            rlim_cur: limit as u64,
                            rlim_max: limit as u64,
                        };
                    }
                }
                if new_limit as usize != 0 {
                    let new_limit = unsafe { (*new_limit).rlim_cur };
                    curr_process.fd_manager.set_limit(new_limit);
                }
            }
            RLIMIT_AS => {
                const USER_MEMORY_LIMIT: usize = 0xffff_ffff;
                if old_limit as usize != 0 {
                    unsafe {
                        *old_limit = RLimit {
                            rlim_cur: USER_MEMORY_LIMIT as u64,
                            rlim_max: USER_MEMORY_LIMIT as u64,
                        };
                    }
                }
            }
            _ => {}
        }
    }
    Ok(0)
}

/// not support
pub fn syscall_getpgid() -> SyscallResult {
    Ok(0)
}

/// # Arguments
/// * `pgid`: usize
pub fn syscall_setpgid(args: [usize; 6]) -> SyscallResult {
    let pgid = args[0];
    info!("not support setpgid, try to set {}", pgid);
    Ok(0)
}

/// 当前不涉及多核情况
pub fn syscall_getpid() -> SyscallResult {
    Ok(current_process().pid() as isize)
}

/// To get the parent process id
pub fn syscall_getppid() -> SyscallResult {
    Ok(current_process().get_parent() as isize)
}

/// # Arguments
/// * `new_mask` - i32
pub fn syscall_umask(args: [usize; 6]) -> SyscallResult {
    let new_mask = args[0] as i32;
    Ok(current_process().fd_manager.set_mask(new_mask) as isize)
}

/// 获取用户 id。在实现多用户权限前默认为最高权限
pub fn syscall_getuid() -> SyscallResult {
    Ok(0)
}

/// 获取有效用户 id，即相当于哪个用户的权限。在实现多用户权限前默认为最高权限
pub fn syscall_geteuid() -> SyscallResult {
    Ok(0)
}

/// 获取用户组 id。在实现多用户权限前默认为最高权限
pub fn syscall_getgid() -> SyscallResult {
    Ok(0)
}

/// 获取有效用户组 id，即相当于哪个用户的权限。在实现多用户权限前默认为最高权限
pub fn syscall_getegid() -> SyscallResult {
    Ok(0)
}

/// 获取当前任务的线程 id
pub fn syscall_gettid() -> SyscallResult {
    Ok(current_task().id().as_u64() as isize)
}

/// Creates a session and sets the process group ID
///
/// The calling process is the leader of the new session
pub fn syscall_setsid() -> SyscallResult {
    let process = current_process();
    let task = current_task();

    let task_id = task.id().as_u64();

    // 当前 process 已经是 process group leader
    if process.pid() == task_id {
        return Err(SyscallError::EPERM);
    }

    // 从当前 process 的 thread group 中移除 calling thread
    process.tasks.lock().retain(|t| t.id().as_u64() != task_id);

    // 新建 process group 并加入
    let new_process = Process::new(
        TaskId::new().as_u64(),
        process.get_parent(),
        Mutex::new(process.memory_set.lock().clone()),
        process.get_heap_bottom(),
        process.fd_manager.fd_table.lock().clone(),
    );

    new_process
        .signal_modules
        .lock()
        .insert(task_id, SignalModule::init_signal(None));

    new_process.tasks.lock().push(task.as_task_ref().clone());
    task.set_leader(true);
    task.set_process_id(new_process.pid());

    // 修改 PID2PC
    PID2PC
        .lock()
        .insert(new_process.pid(), Arc::new(new_process));

    Ok(task_id as isize)
}

/// arch_prc
#[cfg(target_arch = "x86_64")]
/// # Arguments
/// * `code` - usize
/// * `addr` - *mut usize
pub fn syscall_arch_prctl(args: [usize; 6]) -> SyscallResult {
    /*
    #define ARCH_SET_GS			0x1001
    #define ARCH_SET_FS			0x1002
    #define ARCH_GET_FS			0x1003
    #define ARCH_GET_GS			0x1004
    */
    let code = args[0];
    let addr = args[1] as *mut usize;
    match code {
        0x1002 => {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                axhal::arch::write_thread_pointer(*addr);
                // *(read_thread_pointer() as *mut usize) = addr;
            }
            Ok(0)
        }
        0x1003 => {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                *addr = *(axhal::arch::read_thread_pointer() as *mut usize);
            }
            Ok(0)
        }
        0x1001 | 0x1004 => todo!(),
        _ => Err(SyscallError::EINVAL),
    }
    //Ok(0)
}

/// To implement the fork syscall for x86_64
#[cfg(target_arch = "x86_64")]
pub fn syscall_fork() -> SyscallResult {
    warn!("transfer syscall_fork to syscall_clone");
    let args = [1, 0, 0, 0, 0, 0];
    syscall_clone(args)
}

/// prctl
/// # Arguments
/// * `option` - usize
/// * `arg2` - *mut u8
#[cfg(target_arch = "x86_64")]
pub fn syscall_prctl(args: [usize; 6]) -> SyscallResult {
    use core::ptr::slice_from_raw_parts_mut;

    use crate::{PrctlOption, PR_NAME_SIZE};

    let option = args[0];
    let arg2 = args[1] as *mut u8;
    match PrctlOption::try_from(option) {
        Ok(PrctlOption::PR_GET_NAME) => {
            // 获取进程名称。
            let mut process_name = current_task().name().to_string();
            process_name += "\0";
            // [syscall 定义](https://man7.org/linux/man-pages/man2/prctl.2.html)要求 NAME 应该不超过 16 Byte
            process_name.truncate(PR_NAME_SIZE);
            // 把 arg2 转换成可写的 buffer
            if current_process()
                .manual_alloc_for_lazy((arg2 as usize).into())
                .is_ok()
            // 直接访问前需要确保地址已经被分配
            {
                unsafe {
                    let name = &mut *slice_from_raw_parts_mut(arg2, PR_NAME_SIZE);
                    name[..process_name.len()].copy_from_slice(process_name.as_bytes());
                }
                Ok(0)
            } else {
                Err(SyscallError::EINVAL)
            }
        }
        Ok(PrctlOption::PR_SET_NAME) => Ok(0),
        _ => Ok(0),
    }
}
