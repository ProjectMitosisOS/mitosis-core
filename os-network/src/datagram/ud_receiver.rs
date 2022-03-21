use alloc::collections::VecDeque;

use KRdmaKit::qp::UDOp;

use crate::rdma::Err;

use super::msg::UDMsg;
use super::ud::UDDatagram;

/// UDReceiver wraps a UD qp and serves as server-sided message receiver
pub struct UDReceiver<'a> {
    inner: UDDatagram<'a>,
    msg_queues: VecDeque<UDMsg>,
}

impl<'a> UDReceiver<'a> {
    pub fn new(ud: UDDatagram<'a>) -> Self {
        Self {
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

    pub fn to_inner(self) -> (UDDatagram<'a>,  VecDeque<UDMsg>) { 
        (self.inner, self.msg_queues)
    }
}

impl super::Receiver for UDReceiver<'_> {
    type MsgBuf = super::msg::UDMsg;
    type Key = u32;

    /// Post the receive buffer to receive future incoming requests
    /// 
    /// #Arguments
    /// * `buf` - the memory buffer used for receiving future requests
    /// * `key` - the local key of this memory
    fn post_recv_buf(&mut self, buf: Self::MsgBuf, key: Self::Key) -> Result<(), Self::IOResult> {
        let mut op = UDOp::new(&self.inner.ud);
        let res = op.post_recv(buf.get_pa(), key, buf.len());
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
