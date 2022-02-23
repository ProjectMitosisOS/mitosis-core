#![no_std]

extern crate alloc;

mod config;

use krdma_test::*;

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util as log;

#[krdma_main]
fn ctx_init() {
    log::info!("context init");
    log::info!("sample_int: {}", config::get_sample_int());
    log::info!("sample_long: {}", config::get_sample_long());
    log::info!("sample_str: 0x{:x}", config::get_sample_str() as u64);
    log::info!("first charactor in str: {}", unsafe { *config::get_sample_str() as char });
}
