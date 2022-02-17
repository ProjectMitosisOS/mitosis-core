#![no_std]

use krdma_test::*;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use rust_kernel_linux_util as log;

#[krdma_main]
fn test_sample() {
    log::info!("sample test module in raw kernel rdma bindings!");
}
