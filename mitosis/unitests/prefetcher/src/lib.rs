#![no_std]

extern crate alloc;

use mitosis::linux_kernel_module;
use mitosis::log;
use mitosis::prefetcher::*;
use mitosis::startup::*;

use mitosis::remote_mapping::*;
use mitosis::syscalls::*;

use alloc::boxed::Box;
mod my_syscall;
use my_syscall::MySyscallHandler;

#[allow(dead_code)]

struct Module {
    service: SysCallsService<MySyscallHandler>,
}

impl linux_kernel_module::KernelModule for Module {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        // the first part is similar to test_dc_pool
        log::info!("in test prefetcher");

        let mut config: mitosis::Config = Default::default();
        config.set_num_nics_used(1);
        config.set_max_core_cnt(1);
        config.set_init_dc_targets(8);

        start_instance(config).expect("start mitosis instance fail");

        unsafe { assert!(mitosis::get_dc_pool_service_mut().get_dc_qp(0).is_some()) };
        unsafe { assert!(mitosis::get_dc_pool_service_mut().get_dc_qp(12).is_none()) };

        Ok(Self {
            service: SysCallsService::<MySyscallHandler>::new()?,
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
    description: b"A kernel module for testing the prefetcher",
    license: b"GPL"
);
