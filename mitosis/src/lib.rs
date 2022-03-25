#![no_std]
#![feature()]

extern crate alloc;

mod bindings;

pub(crate) use os_network::KRdmaKit;

pub use rust_kernel_linux_util as log;
pub use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

pub const VERSION: usize = 0;

pub mod syscalls;

// The RDMA context used by MITOSIS
pub mod rdma_context;

use alloc::vec::Vec;

#[derive(Debug)]
pub struct Config {
    pub(crate) num_nics_used: usize,
    pub(crate) fallback_threads_num: usize,
    // my machine ID
    pub(crate) machine_id: usize,
    // gid is RDMA address
    pub(crate) peers_gid: Vec<alloc::string::String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_nics_used: 1,
            fallback_threads_num: 2,
            machine_id: 0,
            peers_gid: Vec::new(),
        }
    }
}

impl Config {
    pub fn set_num_nics_used(&mut self, num: usize) -> &mut Self {
        self.num_nics_used = num;
        self
    }

    pub fn set_fallback_num(&mut self, num: usize) -> &mut Self {
        self.fallback_threads_num = num;
        self
    }

    pub fn set_machine_id(&mut self, id: usize) -> &mut Self {
        self.machine_id = id;
        self
    }

    pub fn add_gid(&mut self, gid: alloc::string::String) -> &mut Self {
        self.peers_gid.push(gid);
        self
    }
}

use mitosis_macros::declare_global;

// kernel-space global variables

/***** RDMA-related global data structures */

use KRdmaKit::device::RContext;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::*;

// low-level contexts are directly exposed as global variables 
declare_global!(rdma_driver, alloc::boxed::Box<crate::KRdmaKit::KDriver>);
declare_global!(rdma_contexts, alloc::vec::Vec<crate::RContext<'static>>);
declare_global!(rdma_cm_service, alloc::vec::Vec<crate::RCtrl<'static>>); 

// high-level contexts are abstracted in rdma_context.rs

/***** RDMA-related global data structures ends*/
