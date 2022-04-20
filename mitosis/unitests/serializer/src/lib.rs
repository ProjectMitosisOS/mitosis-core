#![no_std]
#![feature(allocator_api)]
extern crate alloc;

use rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use mitosis::syscalls::*;

mod my_syscall;
use my_syscall::MySyscallHandler;

#[allow(dead_code)]
struct Module {
    service: SysCallsService<MySyscallHandler>,
}

impl linux_kernel_module::KernelModule for Module {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        let mut config: mitosis::Config = Default::default();
        config
            .set_num_nics_used(1)
            .set_rpc_threads(1)
            .set_max_core_cnt(1)
            .set_init_dc_targets(2);
            
        mitosis::startup::start_instance(config);

        Ok(Self {
            service: SysCallsService::<MySyscallHandler>::new()?,
        })
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        mitosis::startup::end_instance();
        log::info!("drop System call modules")
    }
}

linux_kernel_module::kernel_module!(
    Module,
    author: b"xmm",
    description: b"A kernel module for testing system calls",
    license: b"GPL"
);
