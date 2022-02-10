use crate::bytes::BytesMut;
pub trait Device {
    // data for authentication the validity of the operation
    type Key;

    // network address, e.g., IP
    type Location;

    // memory address
    type Address;

    type IOResult<T>;

    fn read(
        &mut self,
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        to: &mut BytesMut,
    ) -> Self::IOResult<()>;

    unsafe fn write(
        &mut self,
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        payload: &BytesMut,
    ) -> Self::IOResult<()>;
}

pub mod local;
// pub mod rdma; 
