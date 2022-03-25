#![no_std]
#![feature(core_intrinsics)]

extern crate alloc;

use mitosis_macros::declare_global;

pub(crate) use os_network::KRdmaKit;

pub use rust_kernel_linux_util as log;
pub use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

pub const VERSION: usize = 0;

pub mod syscalls;

use alloc::vec::Vec;

// TODO: doc how to use mitosis

#[derive(Debug,Clone)]
pub struct Config {
    pub(crate) num_nics_used: usize,
    pub(crate) rpc_threads_num: usize,
    // my machine ID
    pub(crate) machine_id: usize,
    // gid is RDMA address
    pub(crate) peers_gid: Vec<alloc::string::String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_nics_used: 1,
            rpc_threads_num: 2,
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

    pub fn set_rpc_threads(&mut self, num: usize) -> &mut Self {
        self.rpc_threads_num = num;
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

// kernel-space global variables

/***** RDMA-related global data structures */

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::device::RContext;
use KRdmaKit::rust_kernel_rdma_base::*;

/// low-level contexts are directly exposed as global variables
/// These variables can use `start_rdma` and `end_rdma` in rdma_context to
/// create and destroy.
pub mod rdma_context;

declare_global!(rdma_driver, alloc::boxed::Box<crate::KRdmaKit::KDriver>);
declare_global!(rdma_contexts, alloc::vec::Vec<crate::RContext<'static>>);
declare_global!(
    rdma_cm_service,
    alloc::vec::Vec<core::pin::Pin<alloc::boxed::Box<crate::RCtrl<'static>>>>
);

/***** RDMA-related global data structures ends*/

/************ MITOSIS global services ************/

pub mod startup;

/// high-level services are also exposed by global variables.
/// These variables uses the `start_instance` and `end_instance` in startup.rs for initialize.
/// Note, the `start_instance` also calls the `start_rdma`
pub mod rpc_service;
pub mod rpc_handlers;

declare_global!(ud_factories, alloc::vec::Vec<os_network::datagram::ud::UDFactory<'static>>);
declare_global!(dc_factories, alloc::vec::Vec<os_network::rdma::dc::DCFactory<'static>>);

declare_global!(service_rpc, crate::rpc_service::Service);