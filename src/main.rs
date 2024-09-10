//! main module

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
// #![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#![feature(get_mut_unchecked)]

#[macro_use]
mod console;
mod time;
mod sbi;
mod panic;
mod config;
mod mem;
mod sync;
mod task;
mod trap;
mod syscall;
mod loader;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate log;

// #[allow(missing_docs)]
extern crate alloc;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
/// kernel enter point
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello World!");
    mem::init();
    mem::remap_test();
    mem::frame_allocator_test();
    mem::heap_test();
    task::add_initproc();
    println!("after initproc!");
    trap::init();
    trap::enable_timer_interrupt();
    time::set_next_trigger();
    loader::list_apps();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}


fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}
