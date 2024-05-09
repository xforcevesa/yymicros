#![no_std]
#![no_main]

mod utils;
mod sbi;
#[macro_use]
mod console;
mod panic;

use utils::clear_bss;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() ->! {
    clear_bss();
    println!("Hello, world!");
    panic!("Shutdown machine!");
}

