#![no_std]

extern crate alloc;

use mitosis::linux_kernel_module;
use mitosis::log;

use mitosis::remote_mapping::*;
use mitosis::syscalls::*;

use alloc::boxed::Box;
mod my_syscall;
use my_syscall::MySyscallHandler;

fn test_basic() {
    let mut pt = Box::new(RemotePageTable::new());
    log::info!("in test basic page_table: {:?}, is empty {}", pt, pt.is_empty());
        
    // map a single page
    assert!(pt.map(VirtAddr::new(4096), PhysAddr::new(73)).is_none());
    log::info!("in test basic page_table: {:?}, is empty {}", pt, pt.is_empty());

    log::info!("check lookup result {:?}", pt.translate(VirtAddr::new(0xdeadbeaf)));
    log::info!("check lookup result {:?}", pt.translate(VirtAddr::new(4096)));
}

#[allow(dead_code)]
struct Module {
    service : SysCallsService<MySyscallHandler>,
}

impl linux_kernel_module::KernelModule for Module {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        test_basic();
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