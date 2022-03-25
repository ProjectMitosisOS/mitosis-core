#![no_std]

extern crate alloc;

use mitosis::linux_kernel_module;
use mitosis::log;

use mitosis::rdma_context::*;

use krdma_test::*;

#[krdma_main]
fn kmain() {
    log::info!("in test mitosis RDMA context!");

    let mut config : mitosis::Config = Default::default();
    config.set_num_nics_used(2);

    assert!(start_rdma(config).is_some());
}

#[krdma_drop]
fn clean() {
    end_rdma();
}
