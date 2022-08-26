/// A abstraction for read/write remote/local memory
pub trait Device {
    // data for authentication the validity of the operation
    type Key;

    // network address, e.g., IP
    type Location;

    // remote memory address
    type RemoteMemory;

    // local memory address
    type LocalMemory;

    // memory size
    type Size;

    // error type
    type IOResult;

    unsafe fn read(
        &mut self,
        loc: &Self::Location,
        addr: &Self::RemoteMemory,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
        size: &Self::Size,
    ) -> Result<(), Self::IOResult>;

    unsafe fn write(
        &mut self,
        loc: &Self::Location,
        addr: &Self::RemoteMemory,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
        size: &Self::Size,
    ) -> Result<(), Self::IOResult>;
}

pub mod local;
pub mod rdma;
