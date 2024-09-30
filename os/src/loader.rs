//! Loading user applications into memory

use alloc::string::ToString;
use alloc::sync::Arc;
/// Get the total number of applications.
use alloc::{string::String, vec::Vec};
use alloc::format;
use lazy_static::*;

use crate::vfs::{get_file_size, list_dir_by_str, read_file_by_str};

lazy_static! {
    static ref BIN_NAMES: Arc<Vec<String>> = {
        let mut v = Vec::new();
        let mut app_names = list_dir_by_str("/", "/bin").unwrap();
        app_names.sort();
        for app_name in app_names {
            if app_name.ends_with(".elf") {
                // remove ".elf"
                let app_name = &app_name[..app_name.len() - 4];
                v.push(app_name.to_string());
            }
        }
        Arc::new(v)
    };
}

/// get app number
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

/// get bin number
#[allow(unused)]
pub fn get_num_bin() -> usize {
    BIN_NAMES.len()
}

#[allow(unused)]
/// get applications data
pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}

fn get_bin_data_by_id(bin_id: usize) -> Vec<u8> {
    let bin_name = &BIN_NAMES[bin_id];
    let bin_path = format!("/bin/{}.elf", bin_name);
    let bin_size = get_file_size(&bin_path).unwrap();
    let bin_data = read_file_by_str(&bin_path, 0, bin_size as usize).unwrap();
    bin_data
}

#[allow(unused)]
/// get bin data
pub fn get_bin_data(bin_id: usize) -> &'static [u8] {
    let bin_data = &BIN_DATA[bin_id];
    bin_data.as_slice()
}


lazy_static! {
    /// All of bin's data
    static ref BIN_DATA: Arc<Vec<Vec<u8>>> = {
        let num_bin = BIN_NAMES.len();
        let mut v = Vec::new();
        for i in 0..num_bin {
            v.push(get_bin_data_by_id(i));
        }
        Arc::new(v)
    };
}

lazy_static! {
    /// All of app's name
    static ref APP_NAMES: Vec<&'static str> = {
        let num_app = get_num_app();
        extern "C" {
            fn _app_names();
        }
        let mut start = _app_names as usize as *const u8;
        let mut v = Vec::new();
        unsafe {
            for _ in 0..num_app {
                let mut end = start;
                while end.read_volatile() != b'\0' {
                    end = end.add(1);
                }
                let slice = core::slice::from_raw_parts(start, end as usize - start as usize);
                let str = core::str::from_utf8(slice).unwrap();
                v.push(str);
                start = end.add(1);
            }
        }
        v
    };
}

#[allow(unused)]
/// get app data from name
pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    let num_app = get_num_app();
    (0..num_app)
        .find(|&i| APP_NAMES[i] == name)
        .map(get_app_data)
}

#[allow(unused)]
/// get bin data from name
pub fn get_bin_data_by_name(name: &str) -> Option<&'static [u8]> {
    let bin_id = BIN_NAMES.iter().position(|n| n == name)?;
    Some(get_bin_data(bin_id))
}

#[allow(unused)]
/// list all apps
pub fn list_apps() {
    println!("/**** APPS ****");
    for app in APP_NAMES.iter() {
        println!("{}", app);
    }
    println!("**************/");
}

/// list all bins
pub fn list_bins() {
    println!("/**** BINS ****");
    for bin in BIN_NAMES.iter() {
        println!("{}", bin);
    }
    println!("**************/");
}
