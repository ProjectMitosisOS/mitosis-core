use alloc::sync::Arc;

use KRdmaKit::{context::Context, QueuePair};
use KRdmaKit::queue_pairs::DynamicConnectedTarget;

use rust_kernel_linux_util as log;

use core::marker::PhantomData;

pub type DCTarget = DynamicConnectedTarget;

pub struct DCFactory {
    rctx: Arc<Context>,
}

impl DCFactory {
    pub fn new(ctx: &Arc<Context>) -> Self {
        Self { rctx: ctx.clone() }
    }

    pub fn get_context(&self) -> Arc<Context> {
        self.rctx.clone()
    }

    pub fn create_target(&self, key : u64) -> core::option::Option<Arc<DCTarget>> { 
        unimplemented!()
    }
}

pub struct DCConn {
    dc: Arc<QueuePair>,
    watermark : u64,
}

use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;

impl crate::Conn for DCConn {
    type ReqPayload = (); // TODO: Change it to Endpoint+mr+range+signaled+read/write+rkey+raddr
    type CompPayload = ();
    type IOResult = super::Err;

    #[inline]
    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult> {
        compiler_fence(SeqCst);
        unimplemented!()
    }
}

use crate::future::{Async, Future, Poll};

impl Future for DCConn {
    type Output = KRdmaKit::rdma_shim::bindings::ib_wc;
    type Error = super::Err;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        compiler_fence(SeqCst);
        unimplemented!()
    }
}

impl Clone for DCConn {
    /// Clone is fine for DCQP
    /// This is because
    /// 1. it is behind ARC
    /// 2. the raw QP is thread-safe
    fn clone(&self) -> Self {
        Self { 
            dc : self.dc.clone(),   
            watermark : 0,
        }
    }
}

impl crate::conn::Factory for DCFactory {
    type ConnMeta = ();
    type ConnType = DCConn;
    type ConnResult = super::ConnErr;

    fn create(&self, _meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult> {
        unimplemented!()
    }
}

