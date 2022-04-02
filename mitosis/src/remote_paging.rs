use os_network::{rdma::dc::DCConn, Conn};

use crate::kern_wrappers::mm::PhyAddrType;

pub struct RemotePagingService;

impl RemotePagingService {
    /// read the remote physical addr `dst` to `src`, both expressed in physical address
    #[inline]
    pub fn remote_read(
        dst: PhyAddrType,
        src: PhyAddrType,
        sz: usize,
    ) -> Result<(), <os_network::rdma::dc::DCConn<'static> as Conn>::IOResult> {
        let pool_idx = unsafe { crate::bindings::pmem_get_current_cpu() } as usize;
        let dc_qp = unsafe { crate::get_dc_pool_service_mut().get_dc_qp(pool_idx) }
            .expect("failed to get DCQP");
        unimplemented!();
    }
}
