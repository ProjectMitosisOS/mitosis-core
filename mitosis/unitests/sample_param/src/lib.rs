#![no_std]

extern crate alloc;

mod config;

use krdma_test::*;

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util as log;

#[krdma_main]
fn ctx_init() {
    log::info!("context init");
    log::info!("sample_int: {}", config::sample_int::read());
    log::info!("sample_long: {}", config::sample_long::read());
    log::info!("sample_str: 0x{:x}", config::sample_str::read() as u64);
    log::info!("first charactor in str: {}", unsafe { *config::sample_str::read() as char });
}
