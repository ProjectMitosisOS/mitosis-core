use crate::bytes::BytesMut;

pub enum Err {
    /// Timeout error
    Timeout = 0,
    NoID = 1, 
}

pub trait Caller {
    type Addr;
    type IOResult;

    /// #Arguments
    /// * `addr` - Target address, e.g., IP / RDMA's gid
    /// * `id` - Remote function ID 
    /// * request - Request buffer
    /// * reply - Buffer to store the reply
    fn sync_call(
        addr: &Self::Addr,
        id: usize, 
        request: &BytesMut,
        reply: &mut BytesMut,
    ) -> Result<(), (Err, Self::IOResult)>;
}

pub mod datagram_caller;
pub use datagram_caller::DatagramCaller;

pub mod service;
pub use service::Service;
