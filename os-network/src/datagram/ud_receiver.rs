use alloc::{collections::VecDeque, sync::Arc};
use KRdmaKit::{context::Context, DatapathError};

use crate::rpc::GetContext;

use super::msg::UDMsg;
use super::ud::UDDatagram;

/// UDReceiver wraps a UD qp and serves as server-sided message receiver
#[allow(dead_code)]
pub struct UDReceiver {
    qd_hint: usize,
    inner: UDDatagram,
    msg_queues: VecDeque<UDMsg>,
}

/// A wrapper to simplify creating UDReceiver
///
/// Example:
/// ```
/// let receiver = UDReceiverFactory::new()
///                .set_qd_hint(12)
///                .create(ud);
/// ```
///
/// Arguments
/// * qd_hint: the unique id of the targeted UD
#[derive(Default)]
pub struct UDReceiverFactory {
    qd_hint: usize,
}

impl UDReceiverFactory {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_qd_hint(mut self, hint: usize) -> Self {
        self.qd_hint = hint;
        self
    }

    pub fn create(self, ud: UDDatagram) -> UDReceiver {
        UDReceiver::new(ud, self.qd_hint)
    }
}

impl UDReceiver {
    pub fn new(ud: UDDatagram, qd_hint: usize) -> Self {
        Self {
            qd_hint: qd_hint,
            inner: ud,
            msg_queues: VecDeque::new(),
        }
    }

    pub fn to_inner_datagram(self) -> UDDatagram {
        self.inner
    }

    pub fn to_inner_msg_queues(self) -> VecDeque<UDMsg> {
        self.msg_queues
    }

    pub fn to_inner(self) -> (UDDatagram, VecDeque<UDMsg>) {
        (self.inner, self.msg_queues)
    }
}

impl super::Receiver for UDReceiver {
    type MsgBuf = <Self as Future>::Output;
    type IOResult = DatapathError;

    // FIXME: should be configurable
    const HEADER: usize = 40; // GRH header

    const MTU: usize = 4096;

    /// Post the receive buffer to receive future incoming requests
    ///
    /// #Arguments
    /// * `buf` - the memory buffer used for receiving future requests
    fn post_recv_buf(&mut self, buf: Self::MsgBuf) -> Result<(), Self::IOResult> {
        self.inner.get_qp().post_recv(
            buf.get_inner().as_ref(),
            0..buf.len() as u64,
            buf.get_pa(),
        )?;
        self.msg_queues.push_back(buf);
        Ok(())
    }
}

use crate::future::{Async, Future, Poll};

impl Future for UDReceiver {
    type Output = UDMsg;
    type Error = DatapathError;

    /// Poll the underlying completion queue for the UD receive operation
    ///
    /// Return
    /// * If succeed, return the UDMsg poped from internal queue
    /// * If fail, return NotReady or other `DatapathError`
    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        let mut completion = [Default::default(); 1];
        let ret = self.inner.get_qp().poll_recv_cq(&mut completion)?;
        if ret.len() > 0 {
            let msg = self
                .msg_queues
                .pop_front()
                .expect("message queue should contain elements");
            Ok(Async::Ready(msg))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl crate::rpc::GetTransport for UDReceiver {
    type Transport = UDDatagram;

    fn get_transport_mut(&mut self) -> &mut Self::Transport {
        &mut self.inner
    }
}

impl GetContext for UDReceiver {
    type Context = Arc<Context>;

    fn get_context(&self) -> Self::Context {
        self.inner.get_qp().ctx().clone()
    }
}

pub use super::Receiver;
