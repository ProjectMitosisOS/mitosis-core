#![no_std]

extern crate alloc;

use krdma_test::krdma_main;
use mitosis_macros::declare_global;

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util as log;

declare_global!(test_var,u64);

#[krdma_main]
fn main() {
    log::info!("in test global variables");
    unsafe { crate::test_var::init(73) };
    log::info!("test the global variables TEST {}", unsafe { *crate::test_var::get_ref()});
    unsafe { *crate::test_var::get_mut() = 12 };
    log::info!("test the global variables again: TEST {}", unsafe { *crate::test_var::get_ref()});
}
