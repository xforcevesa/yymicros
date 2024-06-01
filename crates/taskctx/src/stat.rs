//! 负责任务时间统计的实现

numeric_enum_macro::numeric_enum! {
    #[repr(i32)]
    #[allow(non_camel_case_types)]
    #[derive(Eq, PartialEq, Debug, Clone, Copy)]
    /// sys_settimer / sys_gettimer 中设定的 which，即计时器类型
    pub enum TimerType {
        /// 表示目前没有任何计时器(不在linux规范中，是os自己规定的)
        NONE = -1,
        /// 统计系统实际运行时间
        REAL = 0,
        /// 统计用户态运行时间
        VIRTUAL = 1,
        /// 统计进程的所有用户态/内核态运行时间
        PROF = 2,
    }
}

impl From<usize> for TimerType {
    fn from(num: usize) -> Self {
        match Self::try_from(num as i32) {
            Ok(val) => val,
            Err(_) => Self::NONE,
        }
    }
}

/// 任务时间统计结构
pub struct TimeStat {
    /// 用户态经过的时间，单位为纳秒
    utime_ns: usize,
    /// 内核态经过的时间，单位为纳秒
    stime_ns: usize,
    /// 进入用户态时标记当前时间戳，用于统计用户态时间
    user_timestamp: usize,
    /// 进入内核态时标记当前时间戳，用于统计内核态时间
    kernel_timestamp: usize,
    /// 计时器类型
    timer_type: TimerType,
    /// 设置下一次触发计时器的区间
    /// 当 timer_remained_us 归零时，**如果 timer_interval_us 非零**，则将其重置为 timer_interval_us 的值；
    /// 否则，则这个计时器不再触发
    timer_interval_ns: usize,
    /// 当前轮次下计数器剩余的时间
    ///
    /// 根据timer_type的种类来进行计算，当归零的时候触发信号，同时进行更新
    timer_remained_ns: usize,

    /// 是否需要发送计时器信号
    pending_timer_signal: bool,
}

impl Default for TimeStat {
    fn default() -> Self {
        Self::new()
    }
}
impl TimeStat {
    /// 新建一个进程时需要初始化时间
    pub fn new() -> Self {
        Self {
            utime_ns: 0,
            stime_ns: 0,
            user_timestamp: 0,
            // 创建新任务时一般都在内核内，所以可以认为进入内核的时间就是当前时间
            kernel_timestamp: 0,
            timer_type: TimerType::NONE,
            timer_interval_ns: 0,
            timer_remained_ns: 0,
            pending_timer_signal: false,
        }
    }

    /// To get the time statistics
    ///
    /// The format is (user time, kernel time) in nanoseconds
    pub fn output(&self) -> (usize, usize) {
        (self.utime_ns, self.stime_ns)
    }

    /// 复位时间统计器
    pub fn reset(&mut self, current_timestamp: usize) {
        self.utime_ns = 0;
        self.stime_ns = 0;
        self.user_timestamp = 0;
        self.kernel_timestamp = current_timestamp;
    }
    /// 从用户态进入内核态，记录当前时间戳，统计用户态时间
    pub fn switch_into_kernel_mode(&mut self, tid: isize, current_timestamp: usize) {
        let now_time_ns = current_timestamp;
        let delta = now_time_ns - self.user_timestamp;
        self.utime_ns += delta;
        self.kernel_timestamp = now_time_ns;
        if self.timer_type != TimerType::NONE {
            self.update_timer(delta, tid);
        };
    }
    /// 从内核态进入用户态，记录当前时间戳，统计内核态时间
    pub fn switch_into_user_mode(&mut self, tid: isize, current_timestamp: usize) {
        // 获取当前时间，单位为纳秒
        let now_time_ns = current_timestamp;
        let delta = now_time_ns - self.kernel_timestamp;
        self.stime_ns += delta;
        self.user_timestamp = now_time_ns;
        if self.timer_type == TimerType::REAL || self.timer_type == TimerType::PROF {
            self.update_timer(delta, tid);
        };
    }
    /// 内核态下，当前任务被切换掉，统计内核态时间
    pub fn swtich_from_old_task(&mut self, tid: isize, current_timestamp: usize) {
        // 获取当前时间，单位为纳秒
        let now_time_ns = current_timestamp;
        let delta = now_time_ns - self.kernel_timestamp;
        self.stime_ns += delta;
        // 需要更新内核态时间戳
        self.kernel_timestamp = now_time_ns;
        if self.timer_type == TimerType::REAL || self.timer_type == TimerType::PROF {
            self.update_timer(delta, tid);
        };
    }
    /// 内核态下，切换到当前任务，更新内核态时间戳
    pub fn switch_to_new_task(&mut self, tid: isize, current_timestamp: usize) {
        // 获取当前时间，单位为纳秒
        let now_time_ns = current_timestamp;
        let delta = now_time_ns - self.kernel_timestamp;
        // 更新时间戳，方便当该任务被切换时统计内核经过的时间
        self.kernel_timestamp = now_time_ns;
        // 注意，对于REAL类型的任务，此时也需要统计经过的时间
        if self.timer_type == TimerType::REAL {
            self.update_timer(delta, tid)
        }
    }

    /// 以微秒形式输出计时器信息
    ///
    /// (计时器周期，当前计时器剩余时间)
    pub fn output_timer_as_us(&self) -> (usize, usize) {
        (self.timer_interval_ns / 1000, self.timer_remained_ns / 1000)
    }

    /// 设定计时器信息
    ///
    /// 若type不为None则返回成功
    pub fn set_timer(
        &mut self,
        timer_interval_ns: usize,
        timer_remained_ns: usize,
        timer_type: usize,
    ) -> bool {
        self.timer_type = timer_type.into();
        self.timer_interval_ns = timer_interval_ns;
        self.timer_remained_ns = timer_remained_ns;
        self.pending_timer_signal = false;
        self.timer_type != TimerType::NONE
    }

    /// 更新计时器，同时判断是否要发出信号
    pub fn update_timer(&mut self, delta: usize, _tid: isize) {
        if self.timer_remained_ns == 0 {
            // 计时器已经结束了
            return;
        }
        if self.timer_remained_ns > delta {
            // 此时计时器还没有结束，直接更新其内容
            self.timer_remained_ns -= delta;
            return;
        }
        // 此时计时器已经结束了，需要准备发出信号
        self.pending_timer_signal = true;
    }

    /// # Return
    /// If the timer has triggered, return the signal number and reset the timer
    /// Otherwise, return None
    ///
    /// Reference:
    /// 1. <https://man7.org/linux/man-pages/man2/setitimer.2.html>
    /// 2. <https://github.com/bminor/musl/blob/master/arch/x86_64/bits/signal.h>
    pub fn check_pending_timer_signal(&mut self) -> Option<usize> {
        if self.pending_timer_signal {
            self.pending_timer_signal = false;
            // 重置计时器
            self.timer_remained_ns = self.timer_interval_ns;
            match self.timer_type {
                TimerType::REAL => Some(14),
                TimerType::VIRTUAL => Some(26),
                TimerType::PROF => Some(27),
                _ => None,
            }
        } else {
            None
        }
    }
}
