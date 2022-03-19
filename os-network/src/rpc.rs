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

pub trait MsgToReq {
    type MsgType;
    type ReqType;

    fn msg_to_req(msg: &Self::MsgType) -> Self::ReqType;
}

#[repr(u8)]
enum ReqType {
    Connect = 0,
    Request = 1,
    Reply = 2,
    DisConenct = 3,
}

pub struct SessionID(usize); 

#[repr(u8)]
pub enum ReplyStatus {
    Ok = 1,       // a success call
    NotExist = 3, // function is not registered in the service
}

#[repr(u64)]
enum RPCMeta {
    Request(SessionID), // session ID
    Reply(ReplyStatus),
}
/// Metadata of RPC messages
pub struct MsgHeader {
    marker: ReqType,
    payload: usize,
    meta: RPCMeta,
}

pub mod service;
pub use service::Service;

pub mod caller;
pub mod hook;

use crate::future::Future;

/// This is a simple wrapper over crate::conn::Conn
/// The reason for doing so is to simplify customization for RPC 
pub trait RPCConn<T: Future = Self>: Future {
    type ReqPayload; // the request format
    type CompPayload = Self::Output;
    type IOResult = Self::Error;

    // post the request to the underlying device
    fn post(&mut self, req: &Self::ReqPayload, signaled : bool) -> Result<(), Self::IOResult>;
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

// concrete implementations based on real transports 
pub mod impls;
