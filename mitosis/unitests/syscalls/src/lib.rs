#![no_std]

extern crate alloc;

use krdma_test::*;

use rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use mitosis::VERSION;

#[krdma_main]
fn syscall_main() {
    log::info!("MITOSIS version {}", VERSION); 
}
