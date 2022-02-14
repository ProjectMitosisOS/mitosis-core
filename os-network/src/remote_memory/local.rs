use core::marker::PhantomData;

use crate::bytes::BytesMut;

pub struct LocalDevice<KeyType, LocationType, IOResult> {
    phantom: PhantomData<KeyType>,
    phantom_1: PhantomData<LocationType>,
    phantom_2 : PhantomData<IOResult>, 
}

impl<KeyType, LocationType, IOResult> LocalDevice<KeyType, LocationType, IOResult> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            phantom_1: PhantomData,
            phantom_2: PhantomData,
        }
    }
}

impl<KeyType, LocationType,IO> super::Device for LocalDevice<KeyType, LocationType,IO> {
    // local memory read/write should succeed or crash
    type Address = u64;
    type Location = LocationType;
    type Key = KeyType;
    type IOResult = IO; 

    /// the addr must be a valid virtual address that can be read by the kernel
    fn read(
        &mut self,
        _loc: &Self::Location,
        addr: &Self::Address,
        _key: &Self::Key,
        to: &mut BytesMut,
    ) -> Result<(), Self::IOResult> {
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
    ) -> Result<(), Self::IOResult> {
        BytesMut::from_raw(*addr as _, payload.len()).copy(payload, 0);
        Ok(())
    }
}
