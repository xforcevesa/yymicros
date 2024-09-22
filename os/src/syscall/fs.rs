//! File and filesystem-related syscalls
use crate::mem::translated_byte_buffer;
use crate::sbi::{console_getchar, console_putchar};
use crate::task::{current_task, current_user_token, suspend_current_and_run_next};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    match fd {
        FD_STDIN => {
            let mut bytes_read: isize = 0;
            let mut buffers = translated_byte_buffer(current_user_token(), buf, len);

            // Ensure that len is non-zero
            assert!(len > 0, "Length should be greater than 0!");

            // println!("len: {}, buffers: {:?}, buffers[0]: {:?}, buffers[0].len(): {}", len, buffers, buffers[0], buffers[0].len());

            for i in 0..len {
                let mut c: usize;
                loop {
                    c = console_getchar(); // Get the next character from the console
                    if c == 0 {
                        // Suspend current task if no input and switch to another task
                        suspend_current_and_run_next();
                        continue;
                    } else {
                        console_putchar(c); // Echo the character back to the console
                        break;
                    }
                }
                let ch = c as u8;
                unsafe {
                    buffers[0].as_mut_ptr().add(i).write_volatile(ch); // Write the character into the buffer
                }
                bytes_read += 1; // Increment the number of bytes read

                if ch == b'\n' || ch == b'\r' || bytes_read >= len as isize {
                    // Stop reading on newline character
                    break;
                }
            }
            // println!("Read Complete");
            bytes_read // Return the number of characters read
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}
