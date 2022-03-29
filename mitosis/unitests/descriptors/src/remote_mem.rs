use core::pin::Pin;
use KRdmaKit::cm::EndPoint;
use KRdmaKit::rust_kernel_rdma_base::*;
use mitosis::kern_wrappers::mm::{PhyAddrType};
use crate::*;


use os_network::{block_on, Conn};
use os_network::rdma::dc::{DCConn};

// TODO: move into network
/// Remote mem_cpy, implemented by one-sided RDMA read.
///
/// Since we are in kernel, both of dst and src address should be physical
#[inline(always)]
pub fn rmem_cpy(dc: &mut DCConn,
                dst: PhyAddrType,
                src: PhyAddrType,
                len: u64,
                lkey: u32,
                remote_meta: &EndPoint) -> isize {
    use os_network::rdma::payload;

    type DCReqPayload = payload::Payload<ib_dc_wr>;
    let point = remote_meta;

    let mut payload = DCReqPayload::default()
        .set_laddr(dst)
        .set_raddr(src)// copy from src into dst
        .set_sz(len as _)
        .set_lkey(lkey)
        .set_rkey(point.mr.get_rkey())
        .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
        .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
        .set_ah(point);

    let mut payload = unsafe { Pin::new_unchecked(&mut payload) };
    os_network::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());

    if dc.post(&payload.as_ref()).is_err() {
        log::error!("unable to post read qp");
        return -1;
    }
    if block_on(dc).is_err() {
        log::error!("polling dc qp with error");
        return -1;
    }
    return 0;
}
