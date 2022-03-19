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

/// Metadata of RPC messages
pub struct ReqHeader {
    id: usize, // function ID
    payload: usize,
}

#[repr(u8)]
pub enum ReplyStatus {
    Ok = 1,       // a success call
    NotExist = 3, // function is not registered in the service
}

pub struct ReplyHeader {
    status: ReplyStatus,
    payload: usize,
}

pub mod service;
pub use service::Service;

pub mod caller;
pub mod hook;
pub mod impls;
