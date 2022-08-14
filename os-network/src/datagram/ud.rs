
use alloc::string::ToString;
use alloc::sync::Arc;

use crate::future::{Async, Future, Poll};

use core::marker::PhantomData;

use KRdmaKit::{context::Context, QueuePair, DatapathError};

// XD: should tuned. maybe use it as a global configure?
pub const MAX_MTU: usize = 4096;
// 40: global routing header (GRH)
pub const MAX_MSG_SZ: usize = MAX_MTU - 40;

pub struct UDFactory {
    rctx: Arc<Context>,
}

impl UDFactory {
    pub fn new(ctx: &Arc<Context>) -> Self {
        Self { rctx: ctx.clone() }
    }

    pub fn get_context(&self) -> Arc<Context> {
        self.rctx.clone()
    }
}

/// Note:
/// - we assume that the datagram is only used by one **thread**
/// UDDatagram wraps a UD qp and serves as client-sided message sender
pub struct UDDatagram {
    pub(crate) ud: Arc<QueuePair>,
    pending : usize,
}

impl UDDatagram {
    pub fn get_qp(&self) -> Arc<QueuePair> {
        self.ud.clone()
    }

    pub fn get_pending(&self) -> usize { 
        self.pending
    }

    pub fn clone(&self) -> Self {
        Self {
            ud: self.ud.clone(),
            pending : self.pending,
        }
    }
}

use crate::rdma::Err;

impl Future for UDDatagram {
    type Output = KRdmaKit::rdma_shim::bindings::ib_wc;
    type Error = Err;

    /// Poll the underlying completion queue for the UD send operation
    /// 
    /// Return
    /// - If succeed, return the ib_wc
    /// - If fail, return NotReady, work-completion-related error or other general error
    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        unimplemented!();
    }
}

impl crate::conn::Conn for UDDatagram
{
    type ReqPayload = (); // TODO: Change it to EndPoint+mr+range+signaled
    type IOResult = DatapathError;
    /// Post the send requests to the underlying qp
    /// 
    /// Return
    /// - If succeed, return Ok(())
    /// - If fail, return a general error
    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult> {
        unimplemented!()
    }
}

impl crate::conn::Factory for UDFactory {
    type ConnMeta = ();
    type ConnType = UDDatagram;
    type ConnResult = Err;

    /// Create a ud qp
    /// 
    /// Return
    /// - If succeed, return a Ok result with UDDatagram
    /// - If fail, return a general error
    fn create(&self, _meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult> {
        unimplemented!();
    }
}

#[derive(Default)]
pub struct UDHyperMeta {
    pub gid: crate::rdma::RawGID,
    pub service_id: u64,
    pub qd_hint: u64,
}

impl crate::conn::MetaFactory for UDFactory {
    // gid, service id, qd hint
    type HyperMeta = UDHyperMeta;

    // ud endpoint, local memory protection key
    type Meta = (KRdmaKit::queue_pairs::DatagramEndpoint, u32);

    type MetaResult = Err;

    /// Note: this function is not optimized, but since it is not on the (important) critical
    /// path of the execution, it is ok to do this
    fn create_meta(&self, meta: Self::HyperMeta) -> Result<Self::Meta, Self::MetaResult> {
        unimplemented!()
    }
}


use core::fmt::{Debug, Formatter};

impl Debug for UDHyperMeta
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f, "UDHyperMeta {{ gid : {}, service_id : {}, qd : {} }}", 
            self.gid.to_string(), self.service_id, self.qd_hint
        )
    }
}