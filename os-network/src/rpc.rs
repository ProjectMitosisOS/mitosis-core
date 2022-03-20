use crate::{bytes::BytesMut, Factory};

pub enum Err {
    /// Timeout error
    Timeout = 0,
    NoID = 1,
}

pub trait Caller {
    type Address;
    type IOResult;

    /// #Arguments
    /// * `addr` - Target address, e.g., IP / RDMA's gid
    /// * `id` - Remote function ID
    /// * `request` - Request buffer
    /// * `reply` - Buffer to store the reply
    fn sync_call(
        addr: &Self::Address,
        id: usize,
        request: &BytesMut,
        reply: &mut BytesMut,
    ) -> Result<(), (Err, Self::IOResult)>;
}

pub mod hook;

pub mod header;
pub mod header_factory;

// modules for registering RPC callbacks
pub mod service;
pub use service::Service;

pub mod caller;

use crate::future::Future;

/// This is a simple wrapper over crate::conn::Conn
/// The reason for doing so is to simplify customization for RPC
pub trait RPCConn<T: Future = Self>: Future {
    type ReqPayload; // the request format
    type CompPayload = Self::Output;
    type IOResult = Self::Error;
    type HyperMeta; 

    // post the request to the underlying device
    fn post(&mut self, req: &Self::ReqPayload, signaled: bool) -> Result<(), Self::IOResult>;
}

/// This is a simple wrapper over crate::conn::Factory
/// The reason for doing so is to simplify customization for RPC
pub trait RPCFactory {
    type ConnMeta;
    type ConnType<'a>: RPCConn
    where
        Self: 'a;
    type ConnResult;

    // create and connect the connection
    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType<'_>, Self::ConnResult>;
}

/// The connection should provide a GenHyperMeta trait, 
/// such that the RPC hook can use it to create a session corresponding to the sender
pub trait GenHyperMeta<F : crate::conn::MetaFactory> { 
    type Args;

    fn generate_hyper(&self, args : &Self::Args) -> F::HyperMeta;
}

// concrete implementations based on real transports
pub mod impls;
