#![no_std]

mod bindings;

use os_network::KRdmaKit;

#[allow(unused_imports)]
pub(crate) use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
#[allow(unused_imports)]
pub(crate) use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util as log;

pub const VERSION: usize = 0;

pub mod syscalls;

// The RDMA context used by MITOSIS
pub mod rdma_context;

#[derive(Debug)]
pub struct Config {
    pub(crate) num_nics_used: usize,
    pub(crate) fallback_threads_num: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_nics_used: 1,
            fallback_threads_num: 2,
        }
    }
}

impl Config { 
    pub fn set_num_nics_used(&mut self, num : usize) -> &mut self { 
        self.num_nics_used = num;
        self
    }

    pub fn set_fallback_num(&mut self, num : usize) -> &mut self { 
        self.fallback_threads_num = num;
        self
    }
}
