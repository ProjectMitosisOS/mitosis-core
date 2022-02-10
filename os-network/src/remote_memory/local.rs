pub struct LocalDevice; 

use crate::bytes::BytesMut;

impl super::Device for LocalDevice { 
    // local memory read/write should succeed or crash
    type IOResult<T> = Result<T,()>;

    type Address = u64; 
    type Location = ();
    type Key = (); 
    
    fn read(
        _loc: &Self::Location,
        addr: &Self::Address,
        _key: &Self::Key,
        to: &mut BytesMut,
    ) -> Self::IOResult<()> { 
        unsafe { to.copy(&BytesMut::from_raw(*addr as _, to.len()), 0) }; 
        Ok(())
    }

    unsafe fn write(
        _loc: Self::Location,
        _addr: Self::Address,
        _key: Self::Key,
        _payload: &BytesMut,
    ) -> Self::IOResult<()> {
        unimplemented!(); 
    }
}