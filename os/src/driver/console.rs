use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
/// print macro implmentation in core
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::driver::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}


#[macro_export]
/// println macro implmentation in core
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::driver::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}