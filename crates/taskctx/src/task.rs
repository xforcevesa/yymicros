#[cfg(feature = "tls")]
use crate::tls::TlsArea;

use crate::{arch::TaskContext, TaskStack, TimeStat};
extern crate alloc;
use alloc::{boxed::Box, string::String};

#[allow(unused_imports)]
use core::{
    cell::UnsafeCell,
    fmt,
    sync::atomic::{AtomicBool, AtomicI32, AtomicU64, AtomicU8, AtomicUsize, Ordering},
};
use memory_addr::{align_up_4k, VirtAddr};

/// A unique identifier for a thread.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TaskId(u64);

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
impl TaskId {
    /// Create a new task ID.
    pub fn new() -> Self {
        Self(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Convert the task ID to a `u64`.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[cfg(feature = "monolithic")]
    /// 清空计数器，为了给单元测试使用
    /// 保留了gc, 主调度，内核进程
    pub fn clear() {
        ID_COUNTER.store(5, Ordering::Relaxed);
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}
/// The possible states of a task.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum TaskState {
    Running = 1,
    Ready = 2,
    Blocked = 3,
    Exited = 4,
}
impl From<u8> for TaskState {
    #[inline]
    fn from(state: u8) -> Self {
        match state {
            1 => Self::Running,
            2 => Self::Ready,
            3 => Self::Blocked,
            4 => Self::Exited,
            _ => unreachable!(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types)]
/// The policy of the scheduler
pub enum SchedPolicy {
    /// The default time-sharing scheduler
    SCHED_OTHER = 0,
    /// The first-in, first-out scheduler
    SCHED_FIFO = 1,
    /// The round-robin scheduler
    SCHED_RR = 2,
    /// The batch scheduler
    SCHED_BATCH = 3,
    /// The idle task scheduler
    SCHED_IDLE = 5,
    /// Unknown scheduler
    SCHED_UNKNOWN,
}

impl From<usize> for SchedPolicy {
    #[inline]
    fn from(policy: usize) -> Self {
        match policy {
            0 => SchedPolicy::SCHED_OTHER,
            1 => SchedPolicy::SCHED_FIFO,
            2 => SchedPolicy::SCHED_RR,
            3 => SchedPolicy::SCHED_BATCH,
            5 => SchedPolicy::SCHED_IDLE,
            _ => SchedPolicy::SCHED_UNKNOWN,
        }
    }
}

impl From<SchedPolicy> for isize {
    #[inline]
    fn from(policy: SchedPolicy) -> Self {
        match policy {
            SchedPolicy::SCHED_OTHER => 0,
            SchedPolicy::SCHED_FIFO => 1,
            SchedPolicy::SCHED_RR => 2,
            SchedPolicy::SCHED_BATCH => 3,
            SchedPolicy::SCHED_IDLE => 5,
            SchedPolicy::SCHED_UNKNOWN => -1,
        }
    }
}

#[derive(Clone, Copy)]
/// The status of the scheduler
pub struct SchedStatus {
    /// The policy of the scheduler
    pub policy: SchedPolicy,
    /// The priority of the scheduler policy
    pub priority: usize,
}

/// The inner task structure used as the minimal unit of scheduling.
pub struct TaskInner {
    id: TaskId,

    name: UnsafeCell<String>,

    /// Whether the task is the idle task
    is_idle: bool,
    /// Whether the task is the initial task
    ///
    /// If the task is the initial task, the kernel will terminate
    /// when the task exits.
    is_init: bool,

    /// The entry point of the task
    ///
    /// For Unikernel, it is the entry point of the spawned task
    ///
    /// For Monolithic Kernel, it points to the function that
    /// will return to the user mode.
    entry: Option<*mut dyn FnOnce()>,

    /// Task state
    state: AtomicU8,

    #[cfg(feature = "preempt")]
    /// Whether the task needs to be rescheduled
    ///
    /// When the time slice is exhausted, it needs to be rescheduled
    need_resched: AtomicBool,
    #[cfg(feature = "preempt")]
    /// The disable count of preemption
    ///
    /// When the task get a lock which need to disable preemption, it
    /// will increase the count. When the lock is released, it will
    /// decrease the count.
    ///
    /// Only when the count is zero, the task can be preempted.
    preempt_disable_count: AtomicUsize,

    #[cfg(feature = "tls")]
    tls: TlsArea,

    exit_code: AtomicI32,

    /// The kernel stack of the task
    kstack: Option<TaskStack>,

    /// The context of the task
    ctx: UnsafeCell<TaskContext>,

    #[cfg(feature = "monolithic")]
    process_id: AtomicU64,

    #[cfg(feature = "monolithic")]
    /// 是否是所属进程下的主线程
    is_leader: AtomicBool,

    // #[cfg(feature = "monolithic")]
    // /// 初始化的trap上下文
    // pub trap_frame: UnsafeCell<TrapFrame>,
    #[cfg(feature = "monolithic")]
    /// the page table token of the process which the task belongs to
    pub page_table_token: UnsafeCell<usize>,

    #[cfg(feature = "monolithic")]
    set_child_tid: AtomicU64,

    #[cfg(feature = "monolithic")]
    clear_child_tid: AtomicU64,

    /// 时间统计, 无论是否为宏内核架构都可能被使用到
    #[allow(unused)]
    time: UnsafeCell<TimeStat>,

    #[cfg(feature = "monolithic")]
    /// TODO: to support the sched_setaffinity
    ///
    /// TODO: move to the upper layer
    pub cpu_set: AtomicU64,

    #[cfg(feature = "monolithic")]
    /// 退出时是否向父进程发送SIG_CHILD
    pub send_sigchld_when_exit: bool,

    #[cfg(feature = "monolithic")]
    /// The scheduler status of the task, which defines the scheduling policy and priority
    pub sched_status: UnsafeCell<SchedStatus>,

    #[cfg(feature = "monolithic")]
    /// Whether the task is a thread which is vforked by another task
    pub is_vforked_child: AtomicBool,
}

unsafe impl Send for TaskInner {}
unsafe impl Sync for TaskInner {}

impl TaskInner {
    /// Gets the ID of the task.
    pub const fn id(&self) -> TaskId {
        self.id
    }

    /// Gets the name of the task.
    pub fn name(&self) -> &str {
        unsafe { (*self.name.get()).as_str() }
    }

    /// Sets the name of the task.
    pub fn set_name(&self, name: &str) {
        unsafe {
            *self.name.get() = String::from(name);
        }
    }

    /// Get a combined string of the task ID and name.
    pub fn id_name(&self) -> alloc::string::String {
        alloc::format!("Task({}, {:?})", self.id.as_u64(), self.name())
    }

    /// 获取内核栈栈顶
    #[inline]
    pub fn get_kernel_stack_top(&self) -> Option<usize> {
        if let Some(kstack) = &self.kstack {
            return Some(kstack.top().as_usize());
        }
        None
    }

    #[cfg(feature = "monolithic")]
    /// Create a new task with the given entry function and stack size.
    pub fn new<F>(
        entry: F,
        name: String,
        stack_size: usize,
        process_id: u64,
        page_table_token: usize,
        sig_child: bool,
        #[cfg(feature = "tls")] tls_area: (usize, usize),
    ) -> TaskInner
    where
        F: FnOnce() + Send + 'static,
    {
        let mut t = Self::new_common(
            TaskId::new(),
            name,
            #[cfg(feature = "tls")]
            tls_area,
        );
        log::debug!("new task: {}", t.id_name());
        let kstack = TaskStack::alloc(align_up_4k(stack_size));

        t.entry = Some(Box::into_raw(Box::new(entry)));

        t.set_sig_child(sig_child);

        t.process_id.store(process_id, Ordering::Release);

        t.page_table_token = UnsafeCell::new(page_table_token);

        t.kstack = Some(kstack);
        if unsafe { &*t.name.get() }.as_str() == "idle" {
            // FIXME: name 现已被用作 prctl 使用的程序名，应另选方式判断 idle 进程
            t.is_idle = true;
        }
        t
    }

    #[cfg(not(feature = "monolithic"))]
    /// Create a new task with the given entry function and stack size.
    pub fn new<F>(
        entry: F,
        name: String,
        stack_size: usize,
        #[cfg(feature = "tls")] tls_area: (usize, usize),
    ) -> TaskInner
    where
        F: FnOnce() + Send + 'static,
    {
        let mut t = Self::new_common(
            TaskId::new(),
            name,
            #[cfg(feature = "tls")]
            tls_area,
        );
        log::debug!("new task: {}", t.id_name());
        let kstack = TaskStack::alloc(align_up_4k(stack_size));

        t.entry = Some(Box::into_raw(Box::new(entry)));

        t.kstack = Some(kstack);
        if unsafe { &*t.name.get() }.as_str() == "idle" {
            // FIXME: name 现已被用作 prctl 使用的程序名，应另选方式判断 idle 进程
            t.is_idle = true;
        }
        t
    }

    /// To init the task context
    ///
    /// # Arguments
    ///
    /// * `entry` - the entry point of the task
    ///
    /// * `kstack_top` - the top of the kernel stack
    ///
    /// * `tls` - the address of the thread local storage
    pub fn init_task_ctx(&mut self, entry: usize, kstack_top: VirtAddr, tls: VirtAddr) {
        self.ctx.get_mut().init(entry, kstack_top, tls);
    }
}

/// Methods for time statistics
impl TaskInner {
    #[inline]
    /// update the time information when the task is switched from user mode to kernel mode
    pub fn time_stat_from_user_to_kernel(&self, current_tick: usize) {
        let time = self.time.get();
        unsafe {
            (*time).switch_into_kernel_mode(self.id.as_u64() as isize, current_tick);
        }
    }

    #[inline]
    /// update the time information when the task is switched from kernel mode to user mode
    pub fn time_stat_from_kernel_to_user(&self, current_tick: usize) {
        let time = self.time.get();
        unsafe {
            (*time).switch_into_user_mode(self.id.as_u64() as isize, current_tick);
        }
    }

    #[inline]
    /// update the time information when the task is switched out
    pub fn time_stat_when_switch_from(&self, current_tick: usize) {
        let time = self.time.get();
        unsafe {
            (*time).swtich_from_old_task(self.id.as_u64() as isize, current_tick);
        }
    }

    #[inline]
    /// update the time information when the task is ready to be switched in
    pub fn time_stat_when_switch_to(&self, current_tick: usize) {
        let time = self.time.get();
        unsafe {
            (*time).switch_to_new_task(self.id.as_u64() as isize, current_tick);
        }
    }

    #[inline]
    /// output the time statistics
    ///
    /// The format is (user time, kernel time) in nanoseconds
    pub fn time_stat_output(&self) -> (usize, usize) {
        let time = self.time.get();
        unsafe { (*time).output() }
    }

    #[inline]
    /// 输出计时器信息
    /// (计时器周期，当前计时器剩余时间)
    /// 单位为us
    pub fn timer_output(&self) -> (usize, usize) {
        let time = self.time.get();
        unsafe { (*time).output_timer_as_us() }
    }

    #[inline]
    /// 设置计时器信息
    ///
    /// 若type不为None则返回成功
    pub fn set_timer(
        &self,
        timer_interval_ns: usize,
        timer_remained_ns: usize,
        timer_type: usize,
    ) -> bool {
        let time = self.time.get();
        unsafe { (*time).set_timer(timer_interval_ns, timer_remained_ns, timer_type) }
    }

    #[inline]
    /// 重置统计时间
    pub fn time_stat_reset(&self, current_tick: usize) {
        let time = self.time.get();
        unsafe {
            (*time).reset(current_tick);
        }
    }
}

#[cfg(feature = "monolithic")]
impl TaskInner {
    /// store the child thread ID at the location pointed to by child_tid in clone args
    pub fn set_child_tid(&self, tid: usize) {
        self.set_child_tid.store(tid as u64, Ordering::Release)
    }

    /// clear (zero) the child thread ID at the location pointed to by child_tid in clone args
    pub fn set_clear_child_tid(&self, tid: usize) {
        self.clear_child_tid.store(tid as u64, Ordering::Release)
    }

    /// get the pointer to the child thread ID
    pub fn get_clear_child_tid(&self) -> usize {
        self.clear_child_tid.load(Ordering::Acquire) as usize
    }

    #[inline]
    /// get the page table token of the process which the task belongs to
    pub fn get_page_table_token(&self) -> usize {
        unsafe { *self.page_table_token.get() }
    }

    #[inline]
    /// force to set the page table token of the process UNSAFELY
    pub fn set_page_table_token(&self, token: usize) {
        unsafe {
            *self.page_table_token.get() = token;
        }
    }

    #[inline]
    /// get the process ID of the task
    pub fn get_process_id(&self) -> u64 {
        self.process_id.load(Ordering::Acquire)
    }

    #[inline]
    /// set the process ID of the task
    pub fn set_process_id(&self, process_id: u64) {
        self.process_id.store(process_id, Ordering::Release);
    }

    // /// 获取内核栈的第一个trap上下文
    // #[inline]
    // pub fn get_first_trap_frame(&self) -> *mut TrapFrame {
    //     if let Some(kstack) = &self.kstack {
    //         return kstack.get_first_trap_frame();
    //     }
    //     unreachable!("get_first_trap_frame: kstack is None");
    // }

    /// set the flag whether the task is the main thread of the process
    pub fn set_leader(&self, is_lead: bool) {
        self.is_leader.store(is_lead, Ordering::Release);
    }

    /// whether the task is the main thread of the process
    pub fn is_leader(&self) -> bool {
        self.is_leader.load(Ordering::Acquire)
    }

    /// 设置CPU set，其中set_size为bytes长度
    pub fn set_cpu_set(&self, mask: usize, set_size: usize, max_cpu_num: usize) {
        let len = if set_size * 4 > max_cpu_num {
            max_cpu_num
        } else {
            set_size * 4
        };
        let now_mask = mask & 1 << ((len) - 1);
        self.cpu_set.store(now_mask as u64, Ordering::Release)
    }

    /// to get the CPU set
    pub fn get_cpu_set(&self) -> usize {
        self.cpu_set.load(Ordering::Acquire) as usize
    }

    /// set the scheduling policy and priority
    pub fn set_sched_status(&self, status: SchedStatus) {
        let prev_status = self.sched_status.get();
        unsafe {
            *prev_status = status;
        }
    }

    /// get the scheduling policy and priority
    pub fn get_sched_status(&self) -> SchedStatus {
        let status = self.sched_status.get();
        unsafe { *status }
    }

    /// get the task context for task switch
    pub fn get_ctx(&self) -> &TaskContext {
        unsafe { self.ctx.get().as_ref().unwrap() }
    }

    /// whether to send SIG_CHILD when the task exits
    pub fn get_sig_child(&self) -> bool {
        self.send_sigchld_when_exit
    }

    /// set whether to send SIG_CHILD when the task exits
    pub fn set_sig_child(&mut self, sig_child: bool) {
        self.send_sigchld_when_exit = sig_child;
    }

    #[cfg(target_arch = "x86_64")]
    /// # Safety
    /// It is unsafe because it may cause undefined behavior if the `fs_base` is not a valid address.
    pub unsafe fn set_tls_force(&self, value: usize) {
        self.ctx.get().as_mut().unwrap().fs_base = value;
    }

    /// To set whether the task will be blocked by a vfork child
    #[inline]
    pub fn set_vfork_child(&self, is_vfork_child: bool) {
        self.is_vforked_child
            .store(is_vfork_child, Ordering::Release);
    }

    /// 获取父进程blocked_by_vfork布尔值
    pub fn is_vfork_child(&self) -> bool {
        self.is_vforked_child.load(Ordering::Acquire)
    }
}

impl TaskInner {
    fn new_common(
        id: TaskId,
        name: String,
        #[cfg(feature = "tls")] tls_area: (usize, usize),
    ) -> Self {
        Self {
            id,
            name: UnsafeCell::new(name),
            is_idle: false,
            is_init: false,
            entry: None,
            state: AtomicU8::new(TaskState::Ready as u8),
            #[cfg(feature = "preempt")]
            need_resched: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            preempt_disable_count: AtomicUsize::new(0),
            exit_code: AtomicI32::new(0),
            kstack: None,
            ctx: UnsafeCell::new(TaskContext::new()),
            #[cfg(feature = "tls")]
            tls: TlsArea::alloc(tls_area.0, tls_area.1),

            time: UnsafeCell::new(TimeStat::new()),

            #[cfg(feature = "monolithic")]
            process_id: AtomicU64::new(0),

            #[cfg(feature = "monolithic")]
            is_leader: AtomicBool::new(false),

            #[cfg(feature = "monolithic")]
            page_table_token: UnsafeCell::new(0),

            #[cfg(feature = "monolithic")]
            set_child_tid: AtomicU64::new(0),

            #[cfg(feature = "monolithic")]
            clear_child_tid: AtomicU64::new(0),

            #[cfg(feature = "monolithic")]
            // 一开始默认都可以运行在每个CPU上
            cpu_set: AtomicU64::new(0),

            #[cfg(feature = "monolithic")]
            sched_status: UnsafeCell::new(SchedStatus {
                policy: SchedPolicy::SCHED_FIFO,
                priority: 1,
            }),

            #[cfg(feature = "monolithic")]
            send_sigchld_when_exit: false,

            #[cfg(feature = "monolithic")]
            is_vforked_child: AtomicBool::new(false),
        }
    }

    /// Creates an "init task" using the current CPU states, to use as the
    /// current task.
    ///
    /// As it is the current task, no other task can switch to it until it
    /// switches out.
    ///
    /// And there is no need to set the `entry`, `kstack` or `tls` fields, as
    /// they will be filled automatically when the task is switches out.
    pub fn new_init(name: String, #[cfg(feature = "tls")] tls_area: (usize, usize)) -> TaskInner {
        let mut t = Self::new_common(
            TaskId::new(),
            name,
            #[cfg(feature = "tls")]
            tls_area,
        );
        t.is_init = true;
        if unsafe { &*t.name.get() }.as_str() == "idle" {
            // FIXME: name 现已被用作 prctl 使用的程序名，应另选方式判断 idle 进程
            t.is_idle = true;
        }
        t
    }

    #[inline]
    /// the state of the task
    pub fn state(&self) -> TaskState {
        self.state.load(Ordering::Acquire).into()
    }

    #[inline]
    /// set the state of the task
    pub fn set_state(&self, state: TaskState) {
        self.state.store(state as u8, Ordering::Release)
    }

    /// Whether the task is running
    #[inline]
    pub fn is_running(&self) -> bool {
        matches!(self.state(), TaskState::Running)
    }

    /// Whether the task is ready to be scheduled
    #[inline]
    pub fn is_ready(&self) -> bool {
        matches!(self.state(), TaskState::Ready)
    }

    /// Whether the task is blocked
    #[inline]
    pub fn is_blocked(&self) -> bool {
        matches!(self.state(), TaskState::Blocked)
    }

    /// Whether the task has been inited
    #[inline]
    pub const fn is_init(&self) -> bool {
        self.is_init
    }

    /// Whether the task is the idle task
    #[inline]
    pub const fn is_idle(&self) -> bool {
        self.is_idle
    }

    /// Set the task waiting for reschedule
    #[inline]
    #[cfg(feature = "preempt")]
    pub fn set_preempt_pending(&self, pending: bool) {
        self.need_resched.store(pending, Ordering::Release)
    }

    /// Get whether the task is waiting for reschedule
    #[inline]
    #[cfg(feature = "preempt")]
    pub fn get_preempt_pending(&self) -> bool {
        self.need_resched.load(Ordering::Acquire)
    }

    /// Whether the task can be preempted
    #[inline]
    #[cfg(feature = "preempt")]
    pub fn can_preempt(&self, current_disable_count: usize) -> bool {
        self.preempt_disable_count.load(Ordering::Acquire) == current_disable_count
    }

    /// Disable the preemption
    #[inline]
    #[cfg(feature = "preempt")]
    pub fn disable_preempt(&self) {
        self.preempt_disable_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Enable the preemption by increasing the disable count
    ///
    /// Only when the count is zero, the task can be preempted
    #[inline]
    #[cfg(feature = "preempt")]
    pub fn enable_preempt(&self) {
        self.preempt_disable_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get the number of preempt disable counter
    #[inline]
    #[cfg(feature = "preempt")]
    pub fn preempt_num(&self) -> usize {
        self.preempt_disable_count.load(Ordering::Acquire)
    }

    /// Get the task context pointer
    ///
    /// # Safety
    ///
    /// The task context pointer is mutable, but it will be accessed by only one task at a time
    #[inline]
    pub const unsafe fn ctx_mut_ptr(&self) -> *mut TaskContext {
        self.ctx.get()
    }

    /// Get the exit code
    #[inline]
    pub fn get_exit_code(&self) -> i32 {
        self.exit_code.load(Ordering::Acquire)
    }

    /// Set the task exit code
    #[inline]
    pub fn set_exit_code(&self, code: i32) {
        self.exit_code.store(code, Ordering::Release)
    }

    /// Get the task entry
    #[inline]
    pub fn get_entry(&self) -> Option<*mut dyn FnOnce()> {
        self.entry
    }

    /// Get the task tls pointer
    #[cfg(feature = "tls")]
    #[inline]
    pub fn get_tls_ptr(&self) -> usize {
        self.tls.tls_ptr() as usize
    }

    /// Reset the task time statistics
    pub fn reset_time_stat(&self, current_timestamp: usize) {
        let time = self.time.get();
        unsafe {
            (*time).reset(current_timestamp);
        }
    }

    /// Check whether the timer triggered
    ///
    /// If the timer has triggered, then reset it and return the signal number
    pub fn check_pending_signal(&self) -> Option<usize> {
        let time = self.time.get();
        unsafe { (*time).check_pending_timer_signal() }
    }
}

impl fmt::Debug for TaskInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TaskInner")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("state", &self.state())
            .finish()
    }
}

impl Drop for TaskInner {
    fn drop(&mut self) {
        log::debug!("task drop: {}", self.id_name());
    }
}
