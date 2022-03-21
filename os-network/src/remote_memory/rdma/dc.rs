use alloc::sync::Arc;

use core::marker::PhantomData;
use core::pin::Pin;

use KRdmaKit::cm::EndPoint;

use crate::conn::Conn;
use crate::rdma::dc::DCConn;

pub struct DCRemoteDevice<'a, LocalMemory> {
    dc: Arc<DCConn<'a>>,
    phantom: PhantomData<LocalMemory>,
}

#[allow(dead_code)]
pub struct DCKeys {
    key_pair: super::MemoryKeys,
    dct_access_key: u32,
}

impl DCKeys {
    pub fn new(lkey: u32, rkey: u32, dct_access_key: u32) -> Self {
        Self {
            key_pair: super::MemoryKeys {
                lkey: lkey,
                rkey: rkey,
            },
            dct_access_key: dct_access_key,
        }
    }
}

impl<'a, LocalMemory> DCRemoteDevice<'a, LocalMemory> {
    pub fn new(dc: Arc<DCConn<'a>>) -> Self {
        Self {
            dc: dc,
            phantom: PhantomData,
        }
    }
}

use KRdmaKit::rust_kernel_rdma_base::{ib_dc_wr, ib_send_flags, ib_wr_opcode};

type DCReqPayload = crate::rdma::payload::Payload<ib_dc_wr>;

impl<LocalMemory> crate::remote_memory::Device for DCRemoteDevice<'_, LocalMemory>
where
    LocalMemory: crate::remote_memory::ToPhys,
{
    // remote memory read/write will succeed or return rdma specific error
    type Address = u64;
    type Location = EndPoint;
    type Key = DCKeys;
    type IOResult = crate::rdma::Err;
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
            .set_lkey(key.key_pair.lkey)
            .set_rkey(key.key_pair.rkey)
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
            .set_ah(loc);

        let mut payload = Pin::new_unchecked(&mut payload);
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
            .set_lkey(key.key_pair.lkey)
            .set_rkey(key.key_pair.rkey)
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
