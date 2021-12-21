#![no_std]

mod manager;
mod bindings;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use linux_kernel_module::{cstr, println};

use manager::MitosisManager;

struct MitosisModule {
    chrdev_registration: linux_kernel_module::chrdev::Registration,
}

impl linux_kernel_module::KernelModule for MitosisModule {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        println!("Start mitosis!");
        let chrdev_registration = linux_kernel_module::chrdev::builder(cstr!("rlibx"), 0..1)?
            .register_device::<MitosisManager>(cstr!("swapx"))
            .build()?;
        Ok(Self {
            chrdev_registration,
        })
    }
}

impl Drop for MitosisModule {
    fn drop(&mut self) {
        println!("Goodbye mitosis!");
    }
}

linux_kernel_module::kernel_module!(
    MitosisModule,
    author: b"xmm",
    description: b"Mitosis",
    license: b"GPL"
);

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}
