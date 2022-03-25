#![no_std]

extern crate alloc;

use mitosis::linux_kernel_module;
use mitosis::log;

use mitosis::startup::*;

use krdma_test::*;

fn test_rpc() { 
    log::info!("in test rpc");
}

#[krdma_test(test_rpc)]
fn init() {
    log::info!("in test mitosis service startups!");

    let mut config : mitosis::Config = Default::default();
    config.set_num_nics_used(2);

    assert!(start_instance(config).is_some());
}

#[krdma_drop]
fn clean() {
    end_instance();
}
