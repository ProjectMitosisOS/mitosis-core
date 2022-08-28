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

/// Read/Write memory in local device
///
/// # Parameters:
/// read:
/// - `addr`: The virtual address of the target memory region to read from
/// - `to`: The target memory region described by `BytesMut` to write to
/// 
/// write:
/// - `addr`: The virtual address of the target memory region to write to
/// - `payload`: The target memory region described by `BytesMut` to read from
///
impl<KeyType, LocationType,IO> super::Device for LocalDevice<KeyType, LocationType,IO> {
    // local memory read/write should succeed or crash
    type RemoteMemory = u64;
    type Location = LocationType;
    type Key = KeyType;
    type IOResult = IO;
    type LocalMemory = BytesMut;
    type Size = ();

    unsafe fn read(
        &mut self,
        _loc: &Self::Location,
        addr: &Self::RemoteMemory,
        _key: &Self::Key,
        to: &mut Self::LocalMemory,
        _size: &Self::Size,
    ) -> Result<(), Self::IOResult> {
        // TODO: shall we check the validity of the in-passing address?
        to.copy(&BytesMut::from_raw(*addr as _, to.len()), 0);
        Ok(())
    }

    /// the addr must be a valid virtual address that can be written by the kernel
    unsafe fn write(
        &mut self,
        _loc: &Self::Location,
        addr: &Self::RemoteMemory,
        _key: &Self::Key,
        payload: &mut Self::LocalMemory,
        _size: &Self::Size,
    ) -> Result<(), Self::IOResult> {
        BytesMut::from_raw(*addr as _, payload.len()).copy(payload, 0);
        Ok(())
    }
}
