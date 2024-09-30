use crate::{mem::UserBuffer, process::suspend_current_and_run_next, sbi::{console_getchar, console_putchar}};

use super::{File, Stat};


/// stdin file for getting chars from console
pub struct Stdin;

/// stdout file for putting chars to console
pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, mut user_buf: UserBuffer) -> usize {
        // Read multiple chars from console
        let mut read_size = 0;
        for slice in user_buf.buffers.iter_mut() {
            let mut i = 0;
            while i < slice.len() {
                let c = console_getchar();
                if c == 0 {
                    suspend_current_and_run_next();
                    // EOF
                    break;
                } else if c == '\n' as usize || c == '\r' as usize {
                    // newline
                    console_putchar('\r' as usize);
                    console_putchar('\n' as usize);
                    break;
                } else if c == '\x7f' as usize {
                    if i > 0 {
                        i -= 1;
                        slice[i] = 0;
                    }
                    // delete char on left
                    console_putchar('\x08' as usize);
                    console_putchar(' ' as usize);
                    console_putchar('\x08' as usize);
                    continue;
                } else {
                    // echo
                    console_putchar(c);
                }
                slice[i] = c as u8;
                i += 1;
            }
            read_size += i;
            // Print the slice as string
            // println!("{}, read_size: {}", core::str::from_utf8(&slice[..i]).unwrap(), read_size);
        }
        read_size
    }
    fn write(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
    fn stat(&self) -> Option<Stat> {
        None
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
    fn stat(&self) -> Option<Stat> {
        None
    }
}