pub trait Device {
    // data for authentication the validity of the operation
    type Key;

    // network address, e.g., IP
    type Location;

    // remote memory address
    type Address;

    // local memory address
    type LocalMemory;

    type IOResult;

    unsafe fn read(
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

/// Any structure implement ToPhys should return
/// its physical address and size
pub trait ToPhys {
    fn to_phys(&self) -> (u64, usize);
}

pub mod local;
pub mod rdma; 
