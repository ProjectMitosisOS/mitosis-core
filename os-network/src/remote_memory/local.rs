pub struct LocalDevice;

use crate::bytes::BytesMut;

impl super::Device for LocalDevice {
    // local memory read/write should succeed or crash
    type IOResult<T> = Result<T, ()>;

    type Address = u64;
    type Location = ();
    type Key = ();

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
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        payload: &BytesMut,
    ) -> Self::IOResult<()> {
        BytesMut::from_raw(*addr as _, payload.len()).copy(payload, 0);
        Ok(())
    }
}
