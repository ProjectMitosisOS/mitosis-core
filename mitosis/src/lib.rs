#![no_std]
#![feature(
core_intrinsics,
allocator_api,
nonnull_slice_from_raw_parts,
alloc_layout_extra,
get_mut_unchecked,
trait_alias
)]

extern crate alloc;
extern crate static_assertions;

use alloc::sync::Arc;
use mitosis_macros::declare_global;

pub use os_network;
pub use os_network::KRdmaKit;

use os_network::KRdmaKit::context::Context;
pub use rust_kernel_linux_util;
pub use rust_kernel_linux_util as log;
pub use rust_kernel_rdma_base::linux_kernel_module;

pub const VERSION: usize = 0;

use alloc::vec::Vec;

pub mod remote_mapping;

// TODO: doc how to use mitosis

pub const MAX_RPC_THREADS_CNT: usize = 10;

// Prefetch page count (fetch by async operation). The real fetched page count would be `PREFETCH_STEP`
pub const PREFETCH_STEP: usize = 1;

pub fn get_calling_cpu_id() -> usize {
    unsafe { crate::bindings::pmem_get_current_cpu() as _ }
}

declare_global!(mac_id, usize);
declare_global!(max_caller_num, usize);

declare_global!(global_locks, alloc::vec::Vec<crate::linux_kernel_module::mutex::LinuxMutex<()>>);

declare_global!(max_cluster_size, usize);

// FIXME: currently, we assume that all machines in the cluster has the
// same number of RNIC attached to it.
declare_global!(max_nics_used, usize);


/// Global configurations of the MITOSIS kernel
#[derive(Debug, Clone)]
pub struct Config {
    pub default_nic_port: u8,

    pub num_nics_used: usize,

    pub rpc_threads_num: usize,

    // my machine ID
    pub machine_id: usize,
    // how many CPU core is available on the machine
    pub max_core_cnt: usize,
    // gid is RDMA address
    pub peers_gid: Vec<alloc::string::String>,

    pub init_dc_targets: usize,

    pub max_cluster_size: usize,

    pub mem_pool_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_nic_port: 1,
            num_nics_used: 1,
            rpc_threads_num: 2,
            machine_id: 0,
            max_core_cnt: 48,
            peers_gid: Vec::new(),
            init_dc_targets: 256,
            max_cluster_size: 128,
            mem_pool_size: 20,
        }
    }
}

impl Config {
    pub fn set_default_nic_port(&mut self, port: u8) -> &mut Self {
        self.default_nic_port = port;
        self
    }

    pub fn set_num_nics_used(&mut self, num: usize) -> &mut Self {
        self.num_nics_used = num;
        self
    }

    pub fn set_rpc_threads(&mut self, num: usize) -> &mut Self {
        assert!(num <= MAX_RPC_THREADS_CNT);
        self.rpc_threads_num = num;
        self
    }

    pub fn set_machine_id(&mut self, id: usize) -> &mut Self {
        self.machine_id = id;
        self
    }

    pub fn get_machine_id(&self) -> usize {
        self.machine_id
    }

    pub fn add_gid(&mut self, gid: alloc::string::String) -> &mut Self {
        self.peers_gid.push(gid);
        self
    }

    pub fn set_max_core_cnt(&mut self, cnt: usize) -> &mut Self {
        self.max_core_cnt = cnt;
        self
    }

    pub fn get_max_core_cnt(&self) -> usize {
        self.max_core_cnt
    }

    pub fn set_init_dc_targets(&mut self, num: usize) -> &mut Self {
        self.init_dc_targets = num;
        self
    }

    pub fn set_mem_pool_size(&mut self, sz: usize) -> &mut Self {
        self.mem_pool_size = sz;
        self
    }
}

// kernel-space global variables

/***** RDMA-related global data structures */

use rust_kernel_rdma_base::*;

/// low-level contexts are directly exposed as global variables
/// These variables can use `start_rdma` and `end_rdma` in rdma_context to
/// create and destroy.
pub mod rdma_context;

declare_global!(rdma_driver, alloc::sync::Arc<crate::KRdmaKit::KDriver>);
declare_global!(rdma_contexts, alloc::vec::Vec<alloc::sync::Arc<crate::KRdmaKit::context::Context>>);

#[inline]
pub unsafe fn get_rdma_context_ref(
    nic_idx: usize,
) -> core::option::Option<&'static Arc<Context>> {
    crate::rdma_contexts::get_ref().get(nic_idx)
}

#[inline]
pub unsafe fn get_max_caller_num()
    -> usize {
    *crate::max_caller_num::get_ref()
}

#[inline]
pub unsafe fn get_mac_id()
    -> usize {
    *crate::mac_id::get_ref()
}

declare_global!(
    rdma_cm_service,
    alloc::vec::Vec<crate::KRdmaKit::comm_manager::CMServer<crate::KRdmaKit::services::UnreliableDatagramAddressService>>
);

declare_global!(
    ud_service,
    alloc::vec::Vec<alloc::sync::Arc<crate::KRdmaKit::services::UnreliableDatagramAddressService>>
);

declare_global!(
    dc_target,
    alloc::vec::Vec<alloc::sync::Arc<crate::KRdmaKit::queue_pairs::DynamicConnectedTarget>>
);

declare_global!(
    dc_target_meta,
    alloc::vec::Vec<crate::KRdmaKit::services::DatagramMeta>
);

#[inline]
pub unsafe fn get_rdma_cm_server_ref(
    nic_idx: usize,
) -> core::option::Option<&'static crate::KRdmaKit::comm_manager::CMServer<crate::KRdmaKit::services::UnreliableDatagramAddressService>> {
    crate::rdma_cm_service::get_ref().get(nic_idx)
}

#[inline]
pub unsafe fn get_ud_service_ref(
    nic_idx: usize,
) -> core::option::Option<&'static alloc::sync::Arc<crate::KRdmaKit::services::UnreliableDatagramAddressService>> {
    crate::ud_service::get_ref().get(nic_idx)
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
    alloc::vec::Vec<alloc::sync::Arc<os_network::datagram::ud::UDFactory>>
);

#[inline]
pub unsafe fn get_ud_factory(
    nic_idx: usize
) -> core::option::Option<&'static alloc::sync::Arc<os_network::datagram::ud::UDFactory>> {
    crate::ud_factories::get_ref().get(nic_idx)
}

#[inline]
pub unsafe fn get_ud_factory_ref(
    nic_idx: usize,
) -> core::option::Option<&'static os_network::datagram::ud::UDFactory> {
    Some(crate::ud_factories::get_ref().get(nic_idx)?.as_ref())
}

declare_global!(
    dc_factories,
    alloc::vec::Vec<os_network::rdma::dc::DCFactory>
);

#[inline]
pub unsafe fn get_dc_factory_ref(
    nic_idx: usize,
) -> core::option::Option<&'static os_network::rdma::dc::DCFactory> {
    crate::dc_factories::get_ref().get(nic_idx)
}

#[inline]
pub fn random_select_dc_factory_on_core(
) -> core::option::Option<&'static os_network::rdma::dc::DCFactory> {
    let pool_idx = unsafe { crate::bindings::pmem_get_current_cpu() } as usize;
    let id = unsafe { pool_idx % crate::dc_factories::get_ref().len() };
    unsafe { crate::dc_factories::get_ref().get(id) }
}

declare_global!(service_rpc, crate::rpc_service::Service);

#[inline]
pub unsafe fn set_service_rpc(arg: crate::rpc_service::Service)
{
    crate::service_rpc::init(arg);
}

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


#[inline]
pub unsafe fn set_rpc_caller_pool(arg: crate::rpc_caller_pool::CallerPool<'static>)
{
    crate::service_caller_pool::init(arg);
}


/// A pool of DCQPs
pub mod dc_pool;
pub mod remote_paging;

declare_global!(dc_pool_service, crate::dc_pool::DCPool);
declare_global!(dc_target_service, crate::dc_pool::DCTargetPool);
declare_global!(access_info_service, crate::dc_pool::AccessInfoPool);


#[cfg(feature = "prefetch")]
type AsyncDCPool = alloc::boxed::Box<lock_bundler::LockBundler<crate::dc_pool::DCPool>>;

#[cfg(feature = "prefetch")]
declare_global!(dc_pool_service_async, crate::AsyncDCPool);

#[inline]
pub unsafe fn get_dc_pool_service_ref() -> &'static crate::dc_pool::DCPool {
    crate::dc_pool_service::get_ref()
}

#[cfg(feature = "prefetch")]
#[inline]
/// The DCQP for async prefetcher is cached in the async servcie pool
/// This design avoid creating DCQP on the fly
pub unsafe fn get_dc_pool_async_service_ref() -> &'static crate::AsyncDCPool {
    crate::dc_pool_service_async::get_ref()
}

#[inline]
pub unsafe fn get_dc_pool_service_mut() -> &'static mut crate::dc_pool::DCPool {
    crate::dc_pool_service::get_mut()
}

#[inline]
pub unsafe fn get_accessinfo_service_mut() -> &'static mut crate::dc_pool::AccessInfoPool {
    crate::access_info_service::get_mut()
}

#[inline]
pub unsafe fn get_dc_target_service_mut() -> &'static mut crate::dc_pool::DCTargetPool {
    crate::dc_target_service::get_mut()
}

declare_global!(
    sp_service,
    crate::shadow_process_service::ShadowProcessService
);

#[inline]
pub unsafe fn get_sps_ref() -> &'static crate::shadow_process_service::ShadowProcessService {
    crate::sp_service::get_ref()
}

#[inline]
pub unsafe fn get_sps_mut() -> &'static mut crate::shadow_process_service::ShadowProcessService {
    crate::sp_service::get_mut()
}

declare_global!(mem_pool, crate::mem_pools::MemPool);

#[inline]
pub unsafe fn get_mem_pool_ref() -> &'static crate::mem_pools::MemPool {
    crate::mem_pool::get_ref()
}

#[inline]
pub unsafe fn get_mem_pool_mut() -> &'static mut crate::mem_pools::MemPool {
    crate::mem_pool::get_mut()
}

// TODO: need add locks
declare_global!(global_pt_cache, crate::remote_pt_cache::RemotePageTableCache);

#[inline]
pub unsafe fn get_pt_cache_ref() -> &'static crate::remote_pt_cache::RemotePageTableCache {
    crate::global_pt_cache::get_ref()
}

#[inline]
pub unsafe fn get_pt_cache_mut() -> &'static mut crate::remote_pt_cache::RemotePageTableCache {
    crate::global_pt_cache::get_mut()
}

// pub mod resume;
pub mod core_syscall_handler;
pub mod syscalls;

pub mod bindings;
pub mod kern_wrappers;

pub mod shadow_process;
pub mod shadow_process_service;

pub mod descriptors;

pub mod mem_pools;

pub mod prefetcher;

pub mod lock_bundler;

pub mod remote_pt_cache;
