#![no_std]
#![feature(core_intrinsics)]

extern crate alloc;

use mitosis_macros::declare_global;

pub use os_network;
pub use os_network::KRdmaKit;

pub use rust_kernel_linux_util;
pub use rust_kernel_linux_util as log;
pub use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

pub const VERSION: usize = 0;

pub mod syscalls;

pub mod bindings;
pub mod kern_wrappers;

pub mod descriptors;

pub mod resume;

use alloc::vec::Vec;

// TODO: doc how to use mitosis

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) num_nics_used: usize,
    pub(crate) rpc_threads_num: usize,
    // my machine ID
    pub(crate) machine_id: usize,
    // how many CPU core is available on the machine
    pub(crate) max_core_cnt: usize,
    // gid is RDMA address
    pub(crate) peers_gid: Vec<alloc::string::String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_nics_used: 1,
            rpc_threads_num: 2,
            machine_id: 0,
            max_core_cnt: 24,
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

    pub fn set_max_core_cnt(&mut self, cnt: usize) -> &mut Self {
        self.max_core_cnt = cnt;
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

#[inline]
pub unsafe fn get_rdma_context_ref(
    nic_idx: usize,
) -> core::option::Option<&'static crate::RContext<'static>> {
    crate::rdma_contexts::get_ref().get(nic_idx)
}

declare_global!(
    rdma_cm_service,
    alloc::vec::Vec<core::pin::Pin<alloc::boxed::Box<crate::RCtrl<'static>>>>
);

#[inline]
pub unsafe fn get_rdma_cm_server_ref(
    nic_idx: usize,
) -> core::option::Option<&'static core::pin::Pin<alloc::boxed::Box<crate::RCtrl<'static>>>> {
    crate::rdma_cm_service::get_ref().get(nic_idx)
}

/***** RDMA-related global data structures ends*/

/************ MITOSIS global services ************/

pub mod startup;

pub mod rpc_handlers;
/// high-level services are also exposed by global variables.
/// These variables uses the `start_instance` and `end_instance` in startup.rs for initialize.
/// Note, the `start_instance` also calls the `start_rdma`
pub mod rpc_service;

declare_global!(
    ud_factories,
    alloc::vec::Vec<os_network::datagram::ud::UDFactory<'static>>
);

#[inline]
pub unsafe fn get_ud_factory_ref(
    nic_idx: usize,
) -> core::option::Option<&'static os_network::datagram::ud::UDFactory<'static>> {
    crate::ud_factories::get_ref().get(nic_idx)
}

declare_global!(
    dc_factories,
    alloc::vec::Vec<os_network::rdma::dc::DCFactory<'static>>
);

#[inline]
pub unsafe fn get_dc_factory_ref(
    nic_idx: usize,
) -> core::option::Option<&'static os_network::rdma::dc::DCFactory<'static>> {
    crate::dc_factories::get_ref().get(nic_idx)
}


declare_global!(service_rpc, crate::rpc_service::Service);

/// A pool of connected RPC clients
pub mod rpc_caller_pool;

declare_global!(
    service_caller_pool,
    crate::rpc_caller_pool::CallerPool<'static>
);
#[inline]
pub unsafe fn get_rpc_caller_pool_ref() -> &'static crate::rpc_caller_pool::CallerPool<'static> {
    crate::service_caller_pool::get_ref()
}

#[inline]
pub unsafe fn get_rpc_caller_pool_mut() -> &'static mut crate::rpc_caller_pool::CallerPool<'static>
{
    crate::service_caller_pool::get_mut()
}

/// A pool of DCQPs 
pub mod dc_pool;

declare_global!(dc_pool_service, crate::dc_pool::DCPool<'static>);

#[inline]
pub unsafe fn get_dc_pool_service_ref() -> &'static crate::dc_pool::DCPool<'static> {
    crate::dc_pool_service::get_ref()
}

#[inline]
pub unsafe fn get_dc_pool_service_mut() -> &'static mut crate::dc_pool::DCPool<'static> {
    crate::dc_pool_service::get_mut()
}

// Descriptor pool, used for container preparation
declare_global!(descriptor_pool, crate::descriptors::DescriptorFactoryService);

#[inline]
pub unsafe fn get_descriptor_pool_ref() -> &'static crate::descriptors::DescriptorFactoryService {
    crate::descriptor_pool::get_ref()
}

#[inline]
pub unsafe fn get_descriptor_pool_mut() -> &'static mut crate::descriptors::DescriptorFactoryService {
    crate::descriptor_pool::get_mut()
}