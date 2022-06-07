#![no_std]

extern crate alloc;

use rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use mitosis::core_syscall_handler::*;
use mitosis::startup::{end_instance, start_instance};
use mitosis::syscalls::*;

/// The module corresponding to the kernel module lifetime 
#[allow(dead_code)]
struct Module {
    service: SysCallsService<MitosisSysCallHandler>,
}

use os_network::block_on;

impl linux_kernel_module::KernelModule for Module {
    /// Called by the kernel upon the kernel module creation
    fn init() -> linux_kernel_module::KernelResult<Self> {

        // Currently, we use a default configuration of MITOSIS
        let mut config: mitosis::Config = Default::default();

        config.set_num_nics_used(2).set_rpc_threads(2).set_init_dc_targets(12);
        
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
    description: b"The kernel module for exposing system calls.",
    license: b"GPL"
);
