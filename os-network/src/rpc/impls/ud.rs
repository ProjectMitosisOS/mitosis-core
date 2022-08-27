use crate::datagram::ud::*;
use crate::future::*;
use crate::rdma::payload::ud::UDReqPayload;

use KRdmaKit::DatagramEndpoint;
use alloc::sync::Arc;

pub struct UDSession {
    meta: Arc<DatagramEndpoint>,
    inner: UDDatagram,
}

impl UDSession {
    #[inline]
    pub fn get_ss_meta(&self) -> &DatagramEndpoint {
        &self.meta
    }

}

impl crate::future::Future for UDSession {
    type Output = <UDDatagram as Future>::Output;
    type Error = <UDDatagram as Future>::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        self.inner.poll()
    }
}

use crate::conn::Conn;

impl super::super::RPCConn for UDSession {
    /// #Argument
    /// * UDMsg: the message to send
    /// * bool : whether to signal the request
    type ReqPayload = crate::msg::UDMsg;
    type HyperMeta = UDHyperMeta;

    #[inline]
    fn post(
        &mut self,
        req: &Self::ReqPayload,
        sz: usize,
        signaled: bool,
    ) -> Result<(), Self::IOResult> {
        let mr = req.get_inner();
        let range = 0..sz as u64;
        let endpoint = self.meta.clone();
        let payload = UDReqPayload::new(mr, range, signaled, endpoint);
        self.inner.post(&payload)
    }

    #[inline]
    fn get_pending_reqs(&self) -> usize {
        self.inner.get_pending()
    }
}

impl super::super::RPCFactory for UDDatagram {
    type ConnMeta = DatagramEndpoint;
    type ConnType = UDSession;

    type ConnResult = crate::rdma::Err;

    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult> {
        let endpoint = meta;
        Ok(UDSession {
            meta: Arc::new(endpoint),
            inner: self.clone(),
        })
    }
}

impl<UDFactory: crate::conn::MetaFactory> super::super::GenHyperMeta<UDFactory>
    for crate::datagram::ud_receiver::UDReceiver
{
    type Args = (alloc::string::String, u64); // gid, service ID

    fn generate_hyper(&self, _args: &Self::Args) -> UDFactory::HyperMeta {
        unimplemented!();
    }
}
