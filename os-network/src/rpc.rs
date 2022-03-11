use crate::bytes::BytesMut;

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
    /// * request - Request buffer
    /// * reply - Buffer to store the reply
    fn sync_call(
        addr: &Self::Address,
        id: usize,
        request: &BytesMut,
        reply: &mut BytesMut,
    ) -> Result<(), (Err, Self::IOResult)>;
}

/// Metadata of RPC messages
pub struct ReqHeader {
    id: usize,
    payload: usize,
}

#[repr(u8)]
enum ReplyStatus {
    Ok = 1,       // a success call
    NotExist = 3, // function is not registered in the service
}

pub struct ReplyHeader {
    status: ReplyStatus,
    payload: usize,
}

pub mod service;
pub use service::Service;

pub mod datagram;
pub use datagram::Caller as DatagramCaller;
