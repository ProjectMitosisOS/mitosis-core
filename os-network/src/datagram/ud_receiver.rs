use alloc::collections::VecDeque;

use KRdmaKit::qp::UDOp;

use crate::rdma::Err;

use super::msg::UDMsg;
use super::ud::UDDatagram;

/// UDReceiver wraps a UD qp and serves as server-sided message receiver
#[allow(dead_code)]
pub struct UDReceiver<'a> {
    qd_hint: usize,
    lkey: u32,
    inner: UDDatagram<'a>,
    msg_queues: VecDeque<UDMsg>,
}

/// A wrapper to simplify creating UDReceiver
///
/// Example:
/// ```
/// let receiver = UDReceiverFactory::new()
///                .set_qd_hint(12)
///                .set_lkey(12)
///                .create(ud);
/// ```
/// 
/// Arguments
/// * lkey : the local memory protection key used throughout the receiver lifetime
/// * qd_hint: the unique id of the targeted UD
#[derive(Default)]
pub struct UDReceiverFactory {
    qd_hint: usize,
    lkey: u32,
}

impl UDReceiverFactory {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_qd_hint(mut self, hint: usize) -> Self {
        self.qd_hint = hint;
        self
    }

    pub fn set_lkey(mut self, lkey: u32) -> Self {
        self.lkey = lkey;
        self
    }

    pub fn create<'a>(self, ud: UDDatagram<'a>) -> UDReceiver<'a> {
        UDReceiver::new(ud, self.qd_hint, self.lkey)
    }
}

impl<'a> UDReceiver<'a> {
    pub fn new(ud: UDDatagram<'a>, qd_hint: usize, key: u32) -> Self {
        Self {
            qd_hint: qd_hint,
            lkey: key,
            inner: ud,
            msg_queues: VecDeque::new(),
        }
    }

    pub fn to_inner_datagram(self) -> UDDatagram<'a> {
        self.inner
    }

    pub fn to_inner_msg_queues(self) -> VecDeque<UDMsg> {
        self.msg_queues
    }

    pub fn to_inner(self) -> (UDDatagram<'a>, VecDeque<UDMsg>) {
        (self.inner, self.msg_queues)
    }
}

impl super::Receiver for UDReceiver<'_> {
    type MsgBuf = <Self as Future>::Output;

    // FIXME: should be configurable
    const HEADER: usize = 40; // GRH header

    const MTU: usize = 4096;

    /// Post the receive buffer to receive future incoming requests
    /// 
    /// #Arguments
    /// * `buf` - the memory buffer used for receiving future requests
    /// * `key` - the local key of this memory
    fn post_recv_buf(&mut self, buf: Self::MsgBuf) -> Result<(), Self::IOResult> {
        let mut op = UDOp::new(&self.inner.ud);
        let res = op.post_recv(buf.get_pa(), self.lkey, buf.len());
        if res.is_err() {
            return Err(Err::Other);
        }
        self.msg_queues.push_back(buf);
        Ok(())
    }
}

use crate::future::{Async, Future, Poll};
use KRdmaKit::rust_kernel_rdma_base::*;

impl Future for UDReceiver<'_> {
    type Output = UDMsg;
    type Error = Err;

    /// Poll the underlying completion queue for the UD receive operation
    /// 
    /// Return
    /// * If succeed, return the UDMsg poped from internal queue
    /// * If fail, return NotReady, work-completion-related error or other general error
    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        let mut wc: ib_wc = Default::default();
        match unsafe { bd_ib_poll_cq(self.inner.ud.get_recv_cq(), 1, &mut wc) } {
            0 => Ok(Async::NotReady),
            1 => {
                if wc.status != ib_wc_status::IB_WC_SUCCESS {
                    return Err(Err::WCErr(wc.status.into()));
                } else {
                    self.msg_queues
                        .pop_front()
                        .ok_or(Err::Empty)
                        .map(|v| Async::Ready(v))
                }
            }
            _ => {
                return Err(Err::Other);
            }
        }
    }
}

pub use super::Receiver;
