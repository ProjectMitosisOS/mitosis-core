#![no_std]

extern crate alloc;

use mitosis::syscalls::*;
use mitosis::linux_kernel_module;
use mitosis::log;

mod my_syscall;
use my_syscall::MySyscallHandler;

#[allow(dead_code)]
struct Module {
    service : SysCallsService<MySyscallHandler>,
}

impl linux_kernel_module::KernelModule for Module {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        Ok(Self { 
            service : SysCallsService::<MySyscallHandler>::new()?
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
    description: b"A kernel module for testing system calls",
    license: b"GPL"
);
