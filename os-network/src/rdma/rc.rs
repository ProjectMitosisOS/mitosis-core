use KRdmaKit::comm_manager::Explorer;
use alloc::sync::Arc;

use KRdmaKit::{QueuePairBuilder, CMError, DatapathError};
use KRdmaKit::{context::Context, QueuePair, QueuePairStatus, ControlpathError};

use super::payload::{LocalMR, RDMAWR, RDMAOp, Signaled};
use super::payload::rc::RCReqPayload;

pub struct RCFactory {
    rctx: Arc<Context>,
}

impl RCFactory {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { rctx: ctx.clone() }
    }

    pub fn get_context(&self) -> Arc<Context> {
        self.rctx.clone()
    }
}

pub enum RCFactoryError {
    ControlpathError(ControlpathError),
    CMError(CMError),
}

impl From<ControlpathError> for RCFactoryError {
    fn from(e: ControlpathError) -> Self {
        Self::ControlpathError(e)
    }
}

impl From<CMError> for RCFactoryError {
    fn from(e: CMError) -> Self {
        Self::CMError(e)
    }
}

impl crate::conn::Factory for RCFactory {
    type ConnMeta = super::ConnMeta;
    type ConnType = RCConn;
    type ConnResult = RCFactoryError;

    /// Create a DC qp
    ///
    /// # Paramters:
    /// - The metadata includes:
    ///     * The gid of the target machine.
    ///     * The service id of the target machine.
    ///     * The port to use in creation of the qp.
    ///
    /// # Errors:
    /// - `CMError`: This error means that there is something wrong when exploring the RDMA subnet admin path record.
    /// - `ControlpathError`: This error means that there is something wrong in communicating with target node.
    ///
    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult> {
        let mut builder = QueuePairBuilder::new(&self.rctx);
        builder
            .allow_remote_rw()
            .allow_remote_atomic()
            .set_port_num(meta.port);
        let client_qp = builder.build_rc()?;
        let explorer = Explorer::new(self.rctx.get_dev_ref());
        let path = unsafe {
            explorer.resolve_inner(meta.service_id, meta.port, meta.gid)
        }?;
        let client_qp = client_qp
            .handshake(meta.service_id, path)?;
        Ok(RCConn {rc: client_qp})
    }
}

// Connection

pub struct RCConn {
    rc: Arc<QueuePair>,
}

impl RCConn {
    pub fn get_status(&self) -> Result<QueuePairStatus, ControlpathError> {
        self.rc.status()
    }

    pub fn get_qp(&self) -> Arc<QueuePair> {
        self.rc.clone()
    }
}

use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;

impl crate::Conn for RCConn {
    type ReqPayload = RCReqPayload;
    type IOResult = DatapathError;

    /// Post the request to the underlying rc qp to perform one-sided read/write operation
    ///
    /// # Parameters:
    /// - The request includes:
    ///     * The read/write flag.
    ///     * The local memory region.
    ///     * The signal flag.
    ///     * The remote address.
    ///     * The remote memory key.
    /// # Errors:
    /// - `DatapathError`: There is something wrong in the data path.
    #[inline]
    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult> {
        compiler_fence(SeqCst);
        match req.get_op() {
            RDMAOp::READ => {
                self.get_qp().post_send_read(
                    req.get_local_mr().as_ref(),
                    req.get_local_mr_range(),
                    req.is_signaled(),
                    req.get_raddr(),
                    req.get_rkey())
            },
            RDMAOp::WRITE => {
                self.get_qp().post_send_write(
                    req.get_local_mr().as_ref(),
                    req.get_local_mr_range(),
                    req.is_signaled(),
                    req.get_raddr(),
                    req.get_rkey())
            },
        }
    }
}

use crate::future::{Async, Future, Poll};

impl Future for RCConn {
    type Output = KRdmaKit::rdma_shim::bindings::ib_wc;
    type Error = DatapathError;

    /// Poll 1 completion from the underlying completion queue in the RC qp
    ///
    /// # Return value:
    /// - `NotReady`: There is nothing in the completion queue.
    /// - A `ib_wc` object: The work completion extracted from the completion queue.
    /// The caller should manually check the status field of the `ib_wc` object.
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
