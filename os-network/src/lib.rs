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
    type ConnType<'a>
    where
        Self::ConnType<'a>: Conn, Self: 'a;
    type ConnResult<T>;

    // create and connect the connection
    fn create<'a>(&'a mut self, meta: Self::ConnMeta) -> Self::ConnResult<Self::ConnType<'a>>
    where
        Self::ConnType<'a>: Conn;
}

// impl the connection as RDMA
pub mod rdma;
