/// Invoke SBI call to put timer
pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

#[allow(unused)]
/// Invoke SBI call to get char
pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

#[allow(unused)]
/// Invoke SBI call to set timer
pub fn set_timer(timer: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::set_timer(timer as u64);
}

pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}

