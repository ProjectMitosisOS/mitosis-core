pub trait Device {
    // data for authentication the validity of the operation
    type Key;

    // network address, e.g., IP
    type Location;

    // remote memory address
    type Address;

    type LocalMemory;

    type IOResult;

    fn read(
        &mut self,
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
    ) -> Result<(), Self::IOResult>;

    unsafe fn write(
        &mut self,
        loc: &Self::Location,
        addr: &Self::Address,
        key: &Self::Key,
        payload: &Self::LocalMemory,
    ) -> Result<(), Self::IOResult>;
}

pub mod local;
pub mod rdma; 
