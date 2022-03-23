#![no_std]

mod bindings;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util as log;

pub(crate) use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

pub const VERSION : usize = 0;

pub mod syscalls;