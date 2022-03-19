use alloc::collections::VecDeque;

use KRdmaKit::qp::UDOp;

use crate::rdma::Err;

use super::msg::UDMsg;
use super::ud::UDDatagram;

pub struct UDReceiver<'a> {
    lkey : u32,
    inner: UDDatagram<'a>,
    msg_queues: VecDeque<UDMsg>,
}

impl<'a> UDReceiver<'a> {
    pub fn new(ud: UDDatagram<'a>, key : u32) -> Self {
        Self {
            lkey : key,
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
    type MsgBuf = <Self as Future>::Output;

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
