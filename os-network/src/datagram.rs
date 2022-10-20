use super::Future;

/// A trait to specificly post (submit) a message buffer to the underlying data structure.
/// The received message is polled from the trait `Future`.
///
pub trait Receiver<T: Future = Self>: Future {
    type IOResult = Self::Error;
    type MsgBuf;

    const HEADER: usize;
    const MTU: usize;

    // post receive buffer to the underlying datagram implementation
    fn post_recv_buf(&mut self, buf: Self::MsgBuf) -> Result<(), Self::IOResult>;
}

pub mod msg;
pub mod ud;
pub mod ud_receiver;
