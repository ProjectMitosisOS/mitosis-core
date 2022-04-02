#![no_std]

extern crate alloc;

use rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use mitosis::syscalls::*;
use mitosis::core_syscall_handler::*;
use mitosis::startup::{end_instance, start_instance};

#[allow(dead_code)]
struct Module {
    service : SysCallsService<MitosisSysCallHandler>,
}

impl linux_kernel_module::KernelModule for Module {

    fn init() -> linux_kernel_module::KernelResult<Self> {

        let mut config: mitosis::Config = Default::default();
        config.set_num_nics_used(2).set_rpc_threads(2);
        assert!(start_instance(config).is_some());

        Ok(Self {
            service : SysCallsService::<MitosisSysCallHandler>::new()?
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
    description: b"A kernel module for testing the core fork functions!",
    license: b"GPL"
);
