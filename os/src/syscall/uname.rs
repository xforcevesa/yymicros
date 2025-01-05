use crate::{
    mem::{read_u8_slice_to_user_buffer, translated_byte_buffer, UserBuffer},
    process::{current_process, current_user_token},
};

#[repr(C)]
pub struct Utsname {
    sysname: [u8; 65],
    nodename: [u8; 65],
    release: [u8; 65],
    version: [u8; 65],
    machine: [u8; 65],
    domainname: [u8; 65],
}

// macro pad_to_64(s: &str) -> [u8; 65]
macro_rules! pad_to_64 {
    ($s:expr) => {{
        let s = $s;
        let mut result = [0u8; 65];
        let bytes = s;
        result[..bytes.len()].copy_from_slice(bytes);
        result
    }};
}

lazy_static::lazy_static! {
    static ref UTSNAME: Utsname = Utsname {
        sysname: pad_to_64!(b"Yosys Microcontroller Operating System\0"),
        nodename: pad_to_64!(b"Yosys\0"),
        release: pad_to_64!(b"0.1\0"),
        version: pad_to_64!(b"0.1\0"),
        machine: pad_to_64!(b"riscv64\0"),
        domainname: pad_to_64!(b"\0"),
    };
}

/// uname syscall
pub fn sys_uname(buf: *mut u8) -> isize {
    let size = core::mem::size_of::<Utsname>();
    let ti_bytes = {
        let ptr = &*UTSNAME as *const Utsname as *const u8;
        let byte_slice = unsafe { core::slice::from_raw_parts(ptr, size) };
        byte_slice
    };
    let mut user_buffer = UserBuffer::new(translated_byte_buffer(
        current_user_token(),
        buf as *const _,
        size,
    ));
    read_u8_slice_to_user_buffer(ti_bytes, &mut user_buffer);
    0
}

#[repr(C)]
pub struct TimesVal {
    tms_utime: usize,
    tms_stime: usize,
    tms_cutime: usize,
    tms_cstime: usize,
}

/// times syscall
pub fn sys_times(buf: *mut TimesVal) -> isize {
    let mut times_val = TimesVal {
        tms_utime: 0,
        tms_stime: 0,
        tms_cutime: 0,
        tms_cstime: 0,
    };
    {
        let process = current_process();
        let inner = process.inner_exclusive_access();
        inner.tasks.iter().for_each(|task| {
            let task = match task {
                Some(task) => task.inner_exclusive_access(),
                None => return,
            };
            times_val.tms_utime += task.time;
            times_val.tms_stime += task.time;
        });
        inner.children.iter().for_each(|child| {
            let child = child.inner_exclusive_access();
            child.tasks.iter().for_each(|task| {
                let task = match task {
                    Some(task) => task.inner_exclusive_access(),
                    None => return,
                };
                times_val.tms_cutime += task.time;
                times_val.tms_cstime += task.time;
            });
        });
    }
    let size = core::mem::size_of::<TimesVal>();
    let ti_bytes = {
        let ptr = &times_val as *const TimesVal as *const u8;
        let byte_slice = unsafe { core::slice::from_raw_parts(ptr, size) };
        byte_slice
    };
    let mut user_buffer = UserBuffer::new(translated_byte_buffer(
        current_user_token(),
        buf as *const _,
        size,
    ));
    read_u8_slice_to_user_buffer(ti_bytes, &mut user_buffer);
    0
}

#[repr(C)]
pub struct LinuxDirent64 {
    pub d_ino: u64,
    pub d_off: u64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [u8; 256],
}

/// getdents64 syscall
pub fn sys_getdents64(fd: usize, buf: *mut u8, nbytes: usize) -> isize {
    let dirent = {
        let process = current_process();
        let inner = process.inner_exclusive_access();
        let file = match inner.fd_table.get(fd) {
            Some(file) => match file {
                Some(f) => f,
                None => return -1,
            },
            None => return -1,
        };
        let dirent = match file.get_dirents() {
            Some(d) => d.as_ptr(),
            None => return -1,
        };
        dirent
    };
    let size = core::mem::size_of::<LinuxDirent64>();
    let ti_bytes = {
        let ptr = dirent as *const u8;
        let byte_slice = unsafe { core::slice::from_raw_parts(ptr, size * nbytes) };
        byte_slice
    };
    let mut user_buffer = UserBuffer::new(translated_byte_buffer(
        current_user_token(),
        buf as *const _,
        size * nbytes,
    ));
    read_u8_slice_to_user_buffer(ti_bytes, &mut user_buffer);
    0
}
