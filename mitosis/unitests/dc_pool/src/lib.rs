#![no_std]

extern crate alloc;

use mitosis::linux_kernel_module;
use mitosis::log;

use mitosis::startup::*;

use krdma_test::*;

#[krdma_main]
fn kmain() {
    log::info!("in test mitosis dc_pool");

    let mut config : mitosis::Config = Default::default();
    config.set_num_nics_used(2);    
    config.set_max_core_cnt(12);

    start_instance(config).expect("start mitosis instance fail");        

    unsafe { assert!(mitosis::get_dc_pool_service_mut().get_dc_qp(0).is_some()) };
    unsafe { assert!(mitosis::get_dc_pool_service_mut().get_dc_qp(12).is_none()) };

    // TODO: shall we do more checks?
}

#[krdma_drop]
fn clean() {
    end_instance();
}
