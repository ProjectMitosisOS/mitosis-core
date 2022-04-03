use os_network::{rdma::dc::DCConn, Conn};

use crate::kern_wrappers::mm::PhyAddrType;

#[derive(Debug)]
struct AccessInfo {
    pub(crate) ah: os_network::rdma::IBAddressHandler,
    pub(crate) rkey: u64,
    pub(crate) dct_num: u64,
    pub(crate) dct_key: u64,
}

impl AccessInfo {
    pub fn new(descriptor: &crate::descriptors::RDMADescriptor) -> core::option::Option<Self> {
        let factory = crate::random_select_dc_factory_on_core()?;
        let ah = os_network::rdma::IBAddressHandlerMeta::create_ah(
            factory.get_context(),
            os_network::rdma::IBAddressHandlerMeta {
                lid: descriptor.lid,
                port_num: descriptor.port_num,
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

impl RemotePagingService {
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

        type DCReqPayload = os_network::rdma::payload::Payload<ib_dc_wr>;

        let mut payload = DCReqPayload::default()
            .set_laddr(dst)
            .set_raddr(src) // copy from src into dst
            .set_sz(len as _)
            .set_lkey(lkey)
            .set_rkey(access_info.rkey)
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
            .set_ah_ptr(unsafe { access_info.ah.get_inner() })
            .set_dc_access_key(access_info.dct_key)
            .set_dc_num(access_info.dct_num);

        let mut payload = unsafe { Pin::new_unchecked(&mut payload) };
        os_network::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());

        // now sending the RDMA request
        dc_qp.post(&payload.as_ref())?;
        
        // wait for the request to complete 
        
    }
}
