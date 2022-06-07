use core::pin::Pin;

use os_network::timeout::TimeoutWRef;
use os_network::{block_on, Conn};

use crate::kern_wrappers::mm::PhyAddrType;
use crate::KRdmaKit::rust_kernel_rdma_base::bindings::*;

#[allow(unused_imports)]
use crate::linux_kernel_module;

pub const TIMEOUT_USEC: i64 = 5000; // 5ms

#[derive(Debug)]
pub struct AccessInfo {
    pub(crate) ah: os_network::rdma::IBAddressHandler,
    pub(crate) rkey: u32,
    pub(crate) dct_num: u32,
    pub(crate) dct_key: usize,
}

impl AccessInfo {
    pub fn new(descriptor: &crate::descriptors::RDMADescriptor) -> core::option::Option<Self> {
        let factory = crate::random_select_dc_factory_on_core()?;
        let ah = os_network::rdma::IBAddressHandlerMeta::create_ah(
            factory.get_context(),
            os_network::rdma::IBAddressHandlerMeta {
                lid: descriptor.lid,
                port_num: descriptor.port_num as _,
                gid: descriptor.gid,
            },
        )?;
        Some(Self {
            ah: ah,
            rkey: descriptor.rkey,
            dct_num: descriptor.dct_num,
            dct_key: descriptor.dct_key,
        })
    }
}

pub struct RemotePagingService;

use crate::remote_mapping::PhysAddr;
use os_network::msg::UDMsg as RMemory;

pub(crate) type DCReqPayload = os_network::rdma::payload::Payload<ib_dc_wr>;

impl RemotePagingService {
    #[inline]
    pub(crate) fn remote_descriptor_fetch(
        d: crate::rpc_handlers::DescriptorLookupReply,
        caller: &mut crate::rpc_caller_pool::UDCaller<'static>,
        session_id: usize,
    ) -> Result<RMemory, <os_network::rdma::dc::DCConn<'static> as Conn>::IOResult> {
        let descriptor_buf = RMemory::new(d.sz, 0);
        let point = caller.get_ss(session_id).unwrap().0.get_ss_meta();

        let pool_idx = unsafe { crate::bindings::pmem_get_current_cpu() } as usize;
        let (dc_qp, lkey) = unsafe { crate::get_dc_pool_service_mut().get_dc_qp_key(pool_idx) }
            .expect("failed to get DCQP");

        let mut payload = DCReqPayload::default()
            .set_laddr(descriptor_buf.get_pa())
            .set_raddr(d.pa) // copy from src into dst
            .set_sz(d.sz as _)
            .set_lkey(*lkey)
            .set_rkey(point.mr.get_rkey())
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
            .set_ah(point);

        let mut payload = unsafe { Pin::new_unchecked(&mut payload) };
        os_network::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());

        // now sending the RDMA request
        dc_qp.post(&payload.as_ref())?;

        // wait for the request to complete
        let mut timeout_dc = TimeoutWRef::new(dc_qp, 10 * TIMEOUT_USEC);
        match block_on(&mut timeout_dc) {
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
        dst: PhyAddrType,
        src: PhyAddrType,
        sz: usize,
        access_info: &AccessInfo,
    ) -> Result<(), <os_network::rdma::dc::DCConn<'static> as Conn>::IOResult> {
        let pool_idx = unsafe { crate::bindings::pmem_get_current_cpu() } as usize;
        let (dc_qp, lkey) = unsafe { crate::get_dc_pool_service_mut().get_dc_qp_key(pool_idx) }
            .expect("failed to get DCQP");

        let mut payload = DCReqPayload::default()
            .set_laddr(dst)
            .set_raddr(PhysAddr::decode(src)) // copy from src into dst
            .set_sz(sz as _)
            .set_lkey(*lkey)
            .set_rkey(access_info.rkey)
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
            .set_ah_ptr(unsafe { access_info.ah.get_inner() })
            .set_dc_access_key(access_info.dct_key as _)
            .set_dc_num(access_info.dct_num);

        // crate::log::debug!("payload update done, key {}", lkey);

        let mut payload = unsafe { Pin::new_unchecked(&mut payload) };
        os_network::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());

        // now sending the RDMA request
        dc_qp.post(&payload.as_ref())?;

        // crate::log::debug!("post dc request done");

        // wait for the request to complete
        let mut timeout_dc = TimeoutWRef::new(dc_qp, TIMEOUT_USEC);
        match block_on(&mut timeout_dc) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.is_elapsed() {
                    // The fallback path? DC cannot distinguish from failures
                    // unimplemented!();
                    crate::log::error!("fatal, timeout on reading the DC QP");
                    Err(os_network::rdma::Err::Timeout)
                    //Ok(())
                } else {
                    Err(e.into_inner().unwrap())
                }
            }
        }
    }
}
