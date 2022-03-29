#![no_std]

extern crate alloc;

use rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use mitosis::syscalls::*;

mod my_syscall;
mod remote_mem;

pub use remote_mem::*;
use my_syscall::MySyscallHandler;
use mitosis::syscalls::SysCallsService;
use krdma_test::*;
use mitosis_macros::{declare_global, declare_module_param};
use mitosis::startup::{end_instance, start_instance};
use linux_kernel_module::c_types::c_uint;
use os_network::Factory;
use os_network::rdma::dc::DCFactory;

declare_module_param!(threads, c_uint);

declare_global!(SERVICE, mitosis::syscalls::SysCallsService<crate::MySyscallHandler>);


declare_global!(GLOBAL_DC_FAC, os_network::rdma::dc::DCFactory<'static>);
declare_global!(GLOBAL_DC, os_network::rdma::dc::DCConn<'static>);

#[inline(always)]
fn setup_instance() {
    // start instance
    let mut config: mitosis::Config = Default::default();
    config.set_num_nics_used(2).set_rpc_threads(2);
    assert!(start_instance(config).is_some());
}

#[krdma_main]
fn init() {
    match SysCallsService::<MySyscallHandler>::new() {
        Ok(s) => {
            unsafe { crate::SERVICE::init(s) };
            setup_instance();
        }
        _ => { log::info!("init kernel module fail!") }
    };

    unsafe {
        let ctx =
            mitosis::get_rpc_caller_pool_ref().get_caller_context(0)
                .unwrap();
        crate::GLOBAL_DC_FAC::init(DCFactory::new(ctx));
        let dc_fac = crate::GLOBAL_DC_FAC::get_ref();
        crate::GLOBAL_DC::init(dc_fac.create(()).unwrap());
    }
}

#[krdma_drop]
fn clean() {
    unsafe {
        SERVICE::drop();
        GLOBAL_DC_FAC::drop();
        GLOBAL_DC::drop();
        end_instance();
    }
    log::info!("drop System call modules");
}
