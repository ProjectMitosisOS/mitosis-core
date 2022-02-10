#![no_std]
#![feature(generic_associated_types, get_mut_unchecked, core_intrinsics)]

//! Manage network connections in the OS

extern crate alloc;

pub mod bytes;
pub mod remote_memory;

pub trait Conn {
    type IOResult<T>;
    type ReqPayload;
    type CompPayload;

    // post the request to the underlying device
    fn post(&mut self, req: &Self::ReqPayload) -> Self::IOResult<()>;

    // poll the completion of the sent request
    fn poll(&mut self) -> Self::IOResult<Self::CompPayload>;
}

pub trait ConnFactory {
    type ConnMeta;
    type ConnType
    where
        Self::ConnType: Conn;
    type ConnResult<T>;

    // create and connect the connection
    fn create(&mut self, meta: Self::ConnMeta) -> Self::ConnResult<Self::ConnType>
    where
        Self::ConnType: Conn;
}

// impl the connection as RDMA
pub mod rdma;
