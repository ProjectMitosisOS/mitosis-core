#![no_std]
#![feature(generic_associated_types, get_mut_unchecked, core_intrinsics)]

//! Manage network connections in the OS

extern crate alloc;

pub mod bytes;
pub mod remote_memory;

pub trait Conn {
    type IOResult; // result of IO rm -
    type ReqPayload; // the request format
    type CompPayload; // the completion (comp) format

    // post the request to the underlying device
    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult>;

    // poll the completion of the sent request, returns immediately
    fn poll(&mut self) -> Result<Self::CompPayload, Self::IOResult>;

    // keep polling until one completion is retrieved
    fn wait_til_comp(&mut self) -> Result<Self::CompPayload, Self::IOResult>;
}

pub trait Datagram {
    type IOResult;
    type AddressHandler;

    fn post_datagram(
        &mut self,
        addr: &Self::AddressHandler,
        msg: &bytes::BytesMut,
    ) -> Result<(), Self::IOResult>;

    // XD: this is not an optimized version,
    // e.g., RDMA's UD will use inlining & doorbell batching for acceleration
    // but currently in MITOSIS's case, implementing these is fine
    // If time permits, we will add the above optimizations
    fn post_recv(&mut self, buf: bytes::BytesMut) -> Result<(), Self::IOResult>;
    fn poll_datagram(&mut self) -> Result<bytes::BytesMut, Self::IOResult>;
}

pub trait ConnFactory {
    type ConnMeta;
    type ConnType<'a>: Conn
    where
        Self: 'a;
    type ConnResult;

    // create and connect the connection
    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType<'_>, Self::ConnResult>;
}

// impl the connection as RDMA
pub mod rdma;
