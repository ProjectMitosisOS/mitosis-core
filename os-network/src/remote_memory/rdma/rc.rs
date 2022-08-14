use alloc::sync::Arc;

use core::marker::PhantomData;
use core::pin::Pin;

use crate::conn::Conn;
use crate::rdma::rc::RCConn;

pub struct RCRemoteDevice<LocalMemory> {
    rc: Arc<RCConn>,
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

impl<LocalMemory> RCRemoteDevice<LocalMemory> {
    pub fn new(rc: Arc<RCConn>) -> Self {
        Self {
            rc: rc,
            phantom: PhantomData,
        }
    }
}

impl<LocalMemory> crate::remote_memory::Device for RCRemoteDevice<LocalMemory>
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
        _loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
    ) -> Result<(), Self::IOResult> {
        unimplemented!()
    }

    unsafe fn write(
        &mut self,
        _loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        payload: &Self::LocalMemory,
    ) -> Result<(), Self::IOResult> {
        unimplemented!()
    }
}
