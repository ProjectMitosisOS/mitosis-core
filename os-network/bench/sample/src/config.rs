use core::{str, slice};
use alloc::string::String;

unsafe fn strlen(ptr: *mut u8) -> usize {
    let mut res: usize = 0;
    loop {
        if *ptr.add(res as usize) == 0 {
            return res;
        }
        res += 1;
    }
}

fn ptr2string(ptr: *mut u8) -> String {
    let s = unsafe {
        str::from_utf8_unchecked(slice::from_raw_parts(ptr, strlen(ptr)))
    };
    String::from(s)
}

#[allow(improper_ctypes)]
extern "C" {
    static SAMPLE: *mut u8;
}

pub fn get_sample() -> String {
    unsafe { ptr2string(SAMPLE) }
}

#[allow(improper_ctypes)]
extern "C" {
    static SAMPLE_LONG: u64;
}

pub fn get_sample_long() -> u64 {
    unsafe { SAMPLE_LONG }
}
