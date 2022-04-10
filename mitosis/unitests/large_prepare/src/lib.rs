#![no_std]

extern crate alloc;

use rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use mitosis::core_syscall_handler::*;
use mitosis::startup::{end_instance, start_instance};
use mitosis::syscalls::*;

#[allow(dead_code)]
struct Module {
    service: SysCallsService<MitosisSysCallHandler>,
}

impl linux_kernel_module::KernelModule for Module {
    fn init() -> linux_kernel_module::KernelResult<Self> {
       let mut config: mitosis::Config = Default::default();
        config.set_num_nics_used(1).set_rpc_threads(1).set_init_dc_targets(1);
        
        assert!(start_instance(config.clone()).is_some());

        Ok(Self {
            service: SysCallsService::<MitosisSysCallHandler>::new()?,
        })        
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        end_instance();
        log::info!("drop System call modules")
    }
}

linux_kernel_module::kernel_module!(
    Module,
    author: b"xmm",
    description: b"A kernel module for testing system calls",
    license: b"GPL"
);
