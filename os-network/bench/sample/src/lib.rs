#![no_std]

extern crate alloc;

mod config;

use krdma_test::*;

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util as log;

fn test() {}

#[krdma_test(test)]
fn ctx_init() {
    log::info!("context init");
    log::info!("sample: {}", config::get_sample());
    log::info!("sample_long: {}", config::get_sample_long());
}
