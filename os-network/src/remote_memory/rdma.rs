use alloc::sync::Arc;

use core::pin::Pin;
use core::marker::PhantomData;

use KRdmaKit::cm::EndPoint;

use crate::conn::Conn;
use crate::rdma::dc::DCConn;
/*
pub struct LocalMemoryBuffer {
    paddr: u64,
    len: usize,
}

impl LocalMemoryBuffer {
    pub unsafe fn new(paddr: u64, len: usize) -> Self {
        Self {
            paddr: paddr,
            len: len,
        }
    }
}

impl super::ToPhys for LocalMemoryBuffer {
    fn to_phys(&self) -> (u64, usize) {
        (self.paddr, self.len)
    }
}
*/
pub struct DCRemoteDevice<'a, LocalMemory> {
    dc: Arc<DCConn<'a>>,
    phantom: PhantomData<LocalMemory>,
}

#[allow(dead_code)]
pub struct DCKey {
    lkey: u32,
    rkey: u32,
    dct_access_key: u32,
}

impl DCKey {
    pub fn new(lkey: u32, rkey: u32, dct_access_key: u32) -> Self {
        Self {
            lkey: lkey,
            rkey: rkey,
            dct_access_key: dct_access_key,
        }
    }
}

impl<'a, LocalMemory> DCRemoteDevice<'a, LocalMemory> {
    pub fn new(dc: Arc<DCConn<'a>>) -> Self {
        Self {
            dc: dc,
            phantom: PhantomData
        }
    }
}

use KRdmaKit::rust_kernel_rdma_base::{ib_send_flags, ib_wr_opcode, ib_dc_wr};
type DCReqPayload = crate::rdma::payload::Payload<ib_dc_wr>;
impl<LocalMemory> super::Device for DCRemoteDevice<'_, LocalMemory>
where
    LocalMemory: super::ToPhys,
{
    // remote memory read/write will succeed or return rdma specific error
    type Address = u64;
    type Location = EndPoint;
    type Key = DCKey;
    type IOResult = super::super::rdma::Err;
    type LocalMemory = LocalMemory;

    unsafe fn read(
        &mut self,
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
    ) -> Result<(), Self::IOResult> {
        let mut payload = DCReqPayload::default()
            .set_laddr(to.to_phys().0)
            .set_raddr(*addr)
            .set_sz(to.to_phys().1)
            .set_lkey(key.lkey)
            .set_rkey(key.rkey)
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
            .set_ah(loc);
        
        let mut payload =  Pin::new_unchecked(&mut payload);
        let dc = Arc::get_mut_unchecked(&mut self.dc);
        crate::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());

        dc.post(&payload.as_ref())?;
        crate::block_on(dc)?;
        Ok(())
    }

    unsafe fn write(
        &mut self,
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        payload: &Self::LocalMemory,
    ) -> Result<(), Self::IOResult> {
        let mut payload = DCReqPayload::default()
            .set_laddr(payload.to_phys().0)
            .set_raddr(*addr)
            .set_sz(payload.to_phys().1)
            .set_lkey(key.lkey)
            .set_rkey(key.rkey)
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_WRITE)
            .set_ah(loc);
        
        let mut payload = Pin::new_unchecked(&mut payload);
        let dc = Arc::get_mut_unchecked(&mut self.dc);
        crate::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());

        dc.post(&payload.as_ref())?;
        crate::block_on(dc)?;
        Ok(())
    }
}
