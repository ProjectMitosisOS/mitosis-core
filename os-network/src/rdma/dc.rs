use alloc::sync::Arc;

use KRdmaKit::queue_pairs::dynamic_connected_transport::DynamicConnectedTargetBuilder;
use KRdmaKit::queue_pairs::DynamicConnectedTarget;
use KRdmaKit::{context::Context, ControlpathError, QueuePair, QueuePairStatus};
use KRdmaKit::{DatapathError, QueuePairBuilder};

use super::payload::dc::DCReqPayload;
use super::payload::{RDMAOp, RDMAWR};

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

    /// Create a DC Target
    ///
    /// # Parameters:
    /// - `key`: A 64-bit dct key to identify the DC Target.
    /// - `port_num`: A number to specified the port to use to create DC Target (The default value should be 1).
    ///
    pub fn create_target(&self, key: u64, port_num: u8) -> core::option::Option<Arc<DCTarget>> {
        let mut builder = DynamicConnectedTargetBuilder::new(&self.rctx);
        builder
            .allow_remote_rw()
            .allow_remote_atomic()
            .set_port_num(port_num);
        builder
            .build_dynamic_connected_target(key)
            .ok()
            .map(|dct| Arc::new(dct))
    }
}

pub struct DCConn {
    dc: Arc<QueuePair>,
    watermark: u64,
}

impl DCConn {
    pub fn get_status(&self) -> Result<QueuePairStatus, ControlpathError> {
        self.dc.status()
    }

    pub fn get_qp(&self) -> Arc<QueuePair> {
        self.dc.clone()
    }
}

use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;

impl crate::Conn for DCConn {
    type ReqPayload = DCReqPayload;
    type CompPayload = ();
    type IOResult = DatapathError;

    /// Post the request to the underlying dc qp to perform one-sided read/write operation
    ///
    /// # Parameters:
    /// - The request includes
    ///     * The read/write flag
    ///     * The endpoint specifying the target node
    ///     * The local memory region
    ///     * The signal flag
    ///     * The remote address
    ///     * The remote memory key
    /// # Errors:
    /// - `DatapathError`: There is something wrong in the data path.
    #[inline]
    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult> {
        compiler_fence(SeqCst);
        match req.get_op() {
            RDMAOp::READ => self.get_qp().post_send_dc_read(
                req.get_endpoint().as_ref(),
                req.get_local_mr().as_ref(),
                req.get_local_mr_range(),
                req.is_signaled(),
                req.get_raddr(),
                req.get_rkey(),
            ),
            RDMAOp::WRITE => self.get_qp().post_send_dc_write(
                req.get_endpoint().as_ref(),
                req.get_local_mr().as_ref(),
                req.get_local_mr_range(),
                req.is_signaled(),
                req.get_raddr(),
                req.get_rkey(),
            ),
        }
    }
}

use crate::future::{Async, Future, Poll};
use crate::rdma::payload::{EndPoint, LocalMR, Signaled};

impl Future for DCConn {
    type Output = KRdmaKit::rdma_shim::bindings::ib_wc;
    type Error = DatapathError;

    /// Poll 1 completion from the underlying completion queue in the DC qp
    ///
    /// # Return value:
    /// - `NotReady`: There is nothing in the completion queue.
    /// - A `ib_wc` object: The work completion extracted from the completion queue.
    /// The caller should check the status field of the `ib_wc` object.
    ///
    /// # Errors:
    /// - `DatapathError`: There is something wrong in polling the completion queue.
    ///
    #[inline]
    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        compiler_fence(SeqCst);
        let mut completion = [Default::default(); 1];
        let ret = self.get_qp().poll_send_cq(&mut completion)?;
        if ret.len() > 0 {
            Ok(Async::Ready(completion[0]))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl Clone for DCConn {
    fn clone(&self) -> Self {
        Self {
            dc: self.dc.clone(),
            watermark: 0,
        }
    }
}

impl crate::conn::Factory for DCFactory {
    type ConnMeta = super::DCCreationMeta;
    type ConnType = DCConn;
    type ConnResult = ControlpathError;

    /// Create a DC qp
    ///
    /// # Paramters:
    /// - The metadata includes
    ///     * The port to use in creation of the qp
    ///
    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult> {
        let mut builder = QueuePairBuilder::new(&self.rctx);
        builder
            .allow_remote_rw()
            .allow_remote_atomic()
            .set_port_num(meta.port);
        let qp = builder.build_dc()?.bring_up_dc()?;
        Ok(DCConn {
            dc: qp,
            watermark: 0,
        })
    }
}
