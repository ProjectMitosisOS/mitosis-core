#![no_std]

mod bindings;

pub(crate) use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
pub(crate) use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util as log;

pub const VERSION : usize = 0;

pub mod syscalls;