#![no_std]

extern crate alloc;

use krdma_test::krdma_main;
use mitosis_macros::declare_module_param; 

declare_module_param!(sample_long, u64);
declare_module_param!(sample_int, u32);
declare_module_param!(sample_str, *mut u8);

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util as log;

#[krdma_main]
fn ctx_init() {
    log::info!("context init");
    log::info!("sample_int: {}", sample_int::read());
    log::info!("sample_long: {}", sample_long::read());
    log::info!("sample_str: 0x{:x}", sample_str::read() as u64);
    log::info!("first charactor in str: {}", unsafe { *sample_str::read() as char });
}
