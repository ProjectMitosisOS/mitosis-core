use alloc::sync::Arc;

use core::marker::PhantomData;
use core::pin::Pin;

use KRdmaKit::queue_pairs::endpoint::DatagramEndpoint;

use crate::conn::Conn;
use crate::rdma::dc::DCConn;

pub struct DCRemoteDevice<LocalMemory> {
    dc: Arc<DCConn>,
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

impl<LocalMemory> DCRemoteDevice<LocalMemory> {
    pub fn new(dc: Arc<DCConn>) -> Self {
        Self {
            dc: dc,
            phantom: PhantomData,
        }
    }
}

impl<LocalMemory> crate::remote_memory::Device for DCRemoteDevice<LocalMemory>
where
    LocalMemory: crate::remote_memory::ToPhys,
{
    // remote memory read/write will succeed or return rdma specific error
    type Address = u64;
    type Location = DatagramEndpoint;
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
        unimplemented!()
    }

    unsafe fn write(
        &mut self,
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        payload: &Self::LocalMemory,
    ) -> Result<(), Self::IOResult> {
        unimplemented!()
    }
}
