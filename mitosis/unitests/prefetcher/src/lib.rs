#![no_std]

extern crate alloc;

use mitosis::linux_kernel_module;
use mitosis::log;

use mitosis::startup::*;
use mitosis::prefetcher::*;

use krdma_test::*;

#[krdma_main]
fn kmain() {
    // the first part is similar to test_dc_pool
    log::info!("in test prefetcher");

    let mut config : mitosis::Config = Default::default();
    config.set_num_nics_used(1);    
    config.set_max_core_cnt(1);
    config.set_init_dc_targets(12);

    start_instance(config).expect("start mitosis instance fail");        

    unsafe { assert!(mitosis::get_dc_pool_service_mut().get_dc_qp(0).is_some()) };
    unsafe { assert!(mitosis::get_dc_pool_service_mut().get_dc_qp(12).is_none()) };

    // now test the prefetcher
    let factory = unsafe { mitosis::get_dc_factory_ref(0).unwrap() };
    let exe = DCAsyncPrefetcher::new(factory).unwrap(); 

    log::info!("prefetch test done"); 
}

#[krdma_drop]
fn clean() {
    end_instance();
}
