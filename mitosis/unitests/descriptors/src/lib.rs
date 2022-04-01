#![no_std]

extern crate alloc;

use rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use mitosis::syscalls::*;
use mitosis::syscalls::SysCallsService;
use krdma_test::*;
use mitosis_macros::{declare_global, declare_module_param};
use mitosis::startup::{end_instance, start_instance};
use linux_kernel_module::c_types::c_uint;

declare_module_param!(threads, c_uint);

declare_global!(SERVICE, mitosis::syscalls::SysCallsService<crate::MitosisHandler>);


#[inline(always)]
fn setup_instance() {
    // start instance
    let mut config: mitosis::Config = Default::default();
    config.set_num_nics_used(2).set_rpc_threads(2);
    assert!(start_instance(config).is_some());
}

#[krdma_main]
fn init() {
    match SysCallsService::<MitosisHandler>::new() {
        Ok(s) => {
            unsafe { crate::SERVICE::init(s) };
            setup_instance();
        }
        _ => { log::info!("init kernel module fail!") }
    };
}

#[krdma_drop]
fn clean() {
    unsafe {
        SERVICE::drop();
        end_instance();
    }
    log::info!("drop System call modules");
}
