use alloc::sync::Arc;
use os_network::KRdmaKit::{DatagramEndpoint, DatapathError};
use os_network::remote_memory::Device;
use os_network::remote_memory::rdma::{DCRemoteDevice, DCKeys};
use os_network::timeout::Timeout;
use os_network::{block_on, Future};

use crate::kern_wrappers::mm::PhyAddrType;
use rust_kernel_rdma_base::bindings::*;

#[allow(unused_imports)]
use crate::linux_kernel_module;

pub const TIMEOUT_USEC: i64 = 1000_000; // 1s

/// Derive copy is rather dangerous
/// This structure is aimed for global usage
#[derive(Debug)]
pub struct AccessInfo {
    pub(crate) access_handler: Arc<crate::KRdmaKit::queue_pairs::DatagramEndpoint>,
    pub(crate) rkey: u32,
    pub(crate) mac_id : usize,
}

impl AccessInfo {
    // FIXME: what if the context of the access info doesn't match the one 
    // in the core_id? 
    pub fn new(descriptor: &crate::descriptors::RDMADescriptor, local_port: u8) -> core::option::Option<Self> {
        let factory = crate::random_select_dc_factory_on_core()?;
        // FIXME: get from global (mapping from gid into ah)
        let endpoint = DatagramEndpoint::new(
            &factory.get_context(),
            local_port,
            descriptor.lid as u32,
            descriptor.gid,
            0, // qpn, meaningless in dct
            0, // qkey, meaningless in dct
            descriptor.dct_num,
            descriptor.dct_key as u64,
        ).ok()?;
        
        Some(Self {
            access_handler: Arc::new(endpoint),
            rkey: descriptor.rkey,
            mac_id : descriptor.mac_id,
        })
    }

    /// create the access info
    /// we first lookup the local CPU's cache
    /// If hit, we will directly return 
    pub fn new_from_cache(mac_id : usize,  des : &crate::descriptors::RDMADescriptor) -> core::option::Option<Self> { 
        /* 
        let pool_idx = unsafe { crate::bindings::pmem_get_current_cpu() } as usize;

        // for sanity check
        if mac_id >= 24 { 
            crate::log::error!("fatal: error machine ID: {}", mac_id);
            return None;
        }
        if unsafe { crate::get_accessinfo_service_mut().query(pool_idx, mac_id).is_none() } { 
            let res = Self::new(des);
            if res.is_none() { 
                crate::log::error!("Failed to create the RDMA address handler @ {} for {}", pool_idx, mac_id);
                return None;
            }
            unsafe { crate::get_accessinfo_service_mut().insert(pool_idx, mac_id, res.unwrap())};
        }
        unsafe { Some(*(crate::get_accessinfo_service_mut().query(pool_idx, mac_id)?)) } */
        unimplemented!();
    }
}

pub struct RemotePagingService;

use crate::remote_mapping::PhysAddr;
use os_network::msg::UDMsg as RMemory;

impl RemotePagingService {
    #[inline]
    pub(crate) fn remote_descriptor_fetch(
        d: crate::rpc_handlers::DescriptorLookupReply,
        caller: &mut crate::rpc_caller_pool::UDCaller,
        session_id: usize,
    ) -> Result<RMemory, <DCRemoteDevice as Future>::Error> {
        let pool_idx = unsafe { crate::bindings::pmem_get_current_cpu() } as usize;
        let dc_qp = unsafe { crate::get_dc_pool_service_mut().get_dc_qp(pool_idx) }
            .expect("failed to get DCQP").clone();

        let descriptor_buf = RMemory::new(d.sz, 0, dc_qp.get_qp().ctx().clone());
        let point = caller.get_ss(session_id).unwrap().0.get_ss_meta();

        // read the descriptor from remote machine
        let mut remote_device = DCRemoteDevice::new(dc_qp);
        unsafe {
            remote_device.read(
                point,
                &d.pa,
                &DCKeys::new(d.rkey),
                &mut descriptor_buf.get_pa(),
                &d.sz)
        }?;
        
        // wait for the request to complete
        let mut timeout_device = Timeout::new(remote_device, 10 * TIMEOUT_USEC);
        match block_on(&mut timeout_device) {
            Ok(_) => Ok(descriptor_buf),
            Err(e) => {
                if e.is_elapsed() {
                    // The fallback path? DC cannot distinguish from failures
                    unimplemented!();
                }
                Err(e.into_inner().unwrap())
            }
        }
    }

    /// read the remote physical addr `dst` to `src`, both expressed in physical address
    #[inline]
    pub fn remote_read(
        mut dst: PhyAddrType,
        src: PhyAddrType,
        sz: usize,
        access_info: &AccessInfo,
    ) -> Result<(), <DCRemoteDevice as Future>::Error> {
        let pool_idx = unsafe { crate::bindings::pmem_get_current_cpu() } as usize;
        let dc_qp = unsafe { crate::get_dc_pool_service_mut().get_dc_qp(pool_idx) }
            .expect("failed to get DCQP").clone();

        // read the requested memory region from remote machine
        let mut remote_device = DCRemoteDevice::new(dc_qp);
        unsafe {
            remote_device.read(
                &access_info.access_handler,
                &PhysAddr::decode(src), // copy from src into dst
                &DCKeys::new(access_info.rkey),
                &mut dst,
                &sz,
            )
        }?;

        // wait for the request to complete
        let mut timeout_device = Timeout::new(remote_device, TIMEOUT_USEC);
        match block_on(&mut timeout_device) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.is_elapsed() {
                    // The fallback path? DC cannot distinguish from failures
                    // unimplemented!();
                    crate::log::error!("fatal, timeout on reading the DC QP");
                    Err(os_network::rdma::Err::DatapathError(DatapathError::TimeoutError))
                } else {
                    Err(e.into_inner().unwrap())
                }
            }
        }
    }
}
