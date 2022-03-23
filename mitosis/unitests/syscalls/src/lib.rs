#![no_std]

extern crate alloc;

use rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use mitosis::syscalls::SysCallsService;

#[allow(dead_code)]
struct Module {
    service : SysCallsService,
}

impl linux_kernel_module::KernelModule for Module {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        Ok(Self { 
            service : SysCallsService::new()?
        })
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        log::info!("drop System call modules")
    }
}

linux_kernel_module::kernel_module!(
    Module,
    author: b"xmm",
    description: b"A kernel module for implement os.swap()",
    license: b"GPL"
);
