use core::marker::PhantomData;

use crate::bytes::BytesMut;

pub struct LocalDevice<KeyType, LocationType> {
    phantom: PhantomData<KeyType>,
    phantom_1: PhantomData<LocationType>,
}

impl<KeyType, LocationType> LocalDevice<KeyType, LocationType> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            phantom_1: PhantomData,
        }
    }
}

impl<KeyType, LocationType> super::Device for LocalDevice<KeyType, LocationType> {
    // local memory read/write should succeed or crash
    type IOResult<T> = crate::rdma::IOResult<T>;

    type Address = u64;
    type Location = LocationType;
    type Key = KeyType;

    /// the addr must be a valid virtual address that can be read by the kernel
    fn read(
        &mut self,
        _loc: &Self::Location,
        addr: &Self::Address,
        _key: &Self::Key,
        to: &mut BytesMut,
    ) -> Self::IOResult<()> {
        // to do: shall we check the validity of the in-passing address?
        unsafe { to.copy(&BytesMut::from_raw(*addr as _, to.len()), 0) };
        Ok(())
    }

    /// the addr must be a valid virtual address that can be written by the kernel
    unsafe fn write(
        &mut self,
        _loc: &Self::Location,
        addr: &Self::Address,
        _key: &Self::Key,
        payload: &BytesMut,
    ) -> Self::IOResult<()> {
        BytesMut::from_raw(*addr as _, payload.len()).copy(payload, 0);
        Ok(())
    }
}
