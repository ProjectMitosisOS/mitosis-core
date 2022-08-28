use alloc::sync::Arc;

use crate::{
    future::{Async, Future, Poll},
    rdma::payload::{ud::UDReqPayload, EndPoint, LocalMR, Signaled},
};

use KRdmaKit::{
    comm_manager::Explorer, context::Context, queue_pairs::endpoint::DatagramEndpointQuerier,
    CMError, ControlpathError, DatapathError, QueuePair, QueuePairBuilder,
};

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
    pending: usize,
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
            pending: self.pending,
        }
    }
}

impl Future for UDDatagram {
    type Output = KRdmaKit::rdma_shim::bindings::ib_wc;
    type Error = DatapathError;

    /// Poll the underlying completion queue for the UD send operation
    ///
    /// Return
    /// - If succeed, return the ib_wc
    /// - If fail, return NotReady, work-completion-related error or other general error
    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        let mut completion = [Default::default()];
        let ret = self.get_qp().poll_send_cq(&mut completion)?;
        if ret.len() > 0 {
            self.pending = 0;
            Ok(Async::Ready(completion[0]))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl crate::conn::Conn for UDDatagram {
    type ReqPayload = UDReqPayload;
    type IOResult = DatapathError;

    /// Post the send requests to the underlying qp
    ///
    /// Return
    /// - If succeed, return Ok(())
    /// - If fail, return a general error
    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult> {
        self.get_qp().post_datagram(
            req.get_endpoint().as_ref(),
            req.get_local_mr().as_ref(),
            req.get_local_mr_range(),
            req.get_local_mr().get_virt_addr(),
            req.is_signaled(),
        )?;
        self.pending += 1;
        Ok(())
    }
}

impl crate::conn::Factory for UDFactory {
    type ConnMeta = UDCreationMeta;
    type ConnType = UDDatagram;
    type ConnResult = ControlpathError;

    /// Create a ud qp
    ///
    /// Return
    /// - If succeed, return a Ok result with UDDatagram
    /// - If fail, return a general error
    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult> {
        let mut builder = QueuePairBuilder::new(&self.get_context());
        builder
            .allow_remote_rw()
            .allow_remote_atomic()
            .set_port_num(meta.port);
        let qp = builder.build_ud()?.bring_up_ud()?;
        Ok(UDDatagram { ud: qp, pending: 0 })
    }
}

#[derive(Default)]
pub struct UDHyperMeta {
    pub gid: KRdmaKit::rdma_shim::bindings::ib_gid,
    pub service_id: u64,
    pub qd_hint: usize,
    pub local_port: u8,
}

#[derive(Default)]
pub struct UDCreationMeta {
    pub port: u8,
}

pub enum UDMetaFactoryError {
    ControlpathError(ControlpathError),
    CMError(CMError),
}

impl From<ControlpathError> for UDMetaFactoryError {
    fn from(e: ControlpathError) -> Self {
        Self::ControlpathError(e)
    }
}

impl From<CMError> for UDMetaFactoryError {
    fn from(e: CMError) -> Self {
        Self::CMError(e)
    }
}

impl crate::conn::MetaFactory for UDFactory {
    type HyperMeta = UDHyperMeta;
    type Meta = KRdmaKit::queue_pairs::DatagramEndpoint;
    type MetaResult = UDMetaFactoryError;

    /// Create endpoint meta data generated from given HyperMeta (gid, service_id, qd_hint, etc.)
    ///
    /// # Errors:
    /// - `CMError`: This error means that there is something wrong when exploring the RDMA subnet admin path record
    /// - `ControlpathError`: This error means that there is something wrong in communicating with target node
    ///
    fn create_meta(&self, meta: Self::HyperMeta) -> Result<Self::Meta, Self::MetaResult> {
        let (gid, service_id, qd_hint, local_port) =
            (meta.gid, meta.service_id, meta.qd_hint, meta.local_port);
        let explorer = Explorer::new(self.get_context().get_dev_ref());
        let path = unsafe { explorer.resolve_inner(service_id, local_port, gid) }?;
        let querier = DatagramEndpointQuerier::create(&self.get_context(), local_port)?;
        let endpoint = querier.query(service_id, qd_hint, path)?;
        Ok(endpoint)
    }
}

use core::fmt::{Debug, Formatter};

impl Debug for UDHyperMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "UDHyperMeta {{ gid : {:?}, service_id : {}, qd : {} }}",
            self.gid, self.service_id, self.qd_hint
        )
    }
}
