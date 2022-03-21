use alloc::sync::Arc;

use core::marker::PhantomData;
use core::pin::Pin;

use crate::conn::Conn;
use crate::rdma::rc::RCConn;

use KRdmaKit::rust_kernel_rdma_base::{ib_rdma_wr, ib_send_flags, ib_wr_opcode};

pub struct RCRemoteDevice<'a, LocalMemory> {
    rc: Arc<RCConn<'a>>,
    phantom: PhantomData<LocalMemory>,
}

pub type RCKeys = super::MemoryKeys;

impl RCKeys {
    pub fn new(lkey: u32, rkey: u32) -> Self {
        Self {
            lkey: lkey,
            rkey: rkey,
        }
    }
}

impl<'a, LocalMemory> RCRemoteDevice<'a, LocalMemory> {
    pub fn new(rc: Arc<RCConn<'a>>) -> Self {
        Self {
            rc: rc,
            phantom: PhantomData,
        }
    }
}

type RCReqPayload = crate::rdma::payload::Payload<ib_rdma_wr>;

impl<LocalMemory> crate::remote_memory::Device for RCRemoteDevice<'_, LocalMemory>
where
    LocalMemory: crate::remote_memory::ToPhys,
{
    // remote memory read/write will succeed or return rdma specific error
    type Address = u64;
    type Location = ();
    type Key = RCKeys;
    type IOResult = crate::rdma::Err;
    type LocalMemory = LocalMemory;

    unsafe fn read(
        &mut self,
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
    ) -> Result<(), Self::IOResult> {
        let mut payload = RCReqPayload::default()
            .set_laddr(to.to_phys().0)
            .set_raddr(*addr)
            .set_sz(to.to_phys().1)
            .set_lkey(key.lkey)
            .set_rkey(key.rkey)
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ);

        let mut payload = Pin::new_unchecked(&mut payload);
        let rc = Arc::get_mut_unchecked(&mut self.rc);
        crate::rdma::payload::Payload::<ib_rdma_wr>::finalize(payload.as_mut());

        rc.post(&payload.as_ref())?;
        crate::block_on(rc)?;
        Ok(())
    }

    unsafe fn write(
        &mut self,
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        payload: &Self::LocalMemory,
    ) -> Result<(), Self::IOResult> {
        let mut payload = RCReqPayload::default()
            .set_laddr(payload.to_phys().0)
            .set_raddr(*addr)
            .set_sz(payload.to_phys().1)
            .set_lkey(key.lkey)
            .set_rkey(key.rkey)
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_WRITE);

        let mut payload = Pin::new_unchecked(&mut payload);
        let rc = Arc::get_mut_unchecked(&mut self.rc);
        crate::rdma::payload::Payload::<ib_rdma_wr>::finalize(payload.as_mut());

        rc.post(&payload.as_ref())?;
        crate::block_on(rc)?;
        Ok(())
    }
}
