use crate::datagram::ud::*;
use crate::future::*;

use KRdmaKit::cm::EndPoint;
use KRdmaKit::rust_kernel_rdma_base::*;

pub struct UDSession<'a> {
    meta: EndPoint,
    inner: UDDatagram<'a>,
    key: u32,
}

impl<'a> crate::future::Future for UDSession<'a> {
    type Output = <UDDatagram<'a> as Future>::Output;
    type Error = <UDDatagram<'a> as Future>::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        self.inner.poll()
    }
}

use crate::conn::Conn;
use core::pin::Pin;

impl super::super::RPCConn for UDSession<'_> {
    /// #Argument
    /// * UDMsg: the message to send
    /// * bool : whether to signal the request
    type ReqPayload = crate::msg::UDMsg;
    type HyperMeta = ();

    #[inline]
    fn post(&mut self, req: &Self::ReqPayload, signaled: bool) -> Result<(), Self::IOResult> {
        let mut send_req = req
            .to_ud_wr(&self.meta)
            .set_send_flags(match signaled {
                true => ib_send_flags::IB_SEND_SIGNALED,
                false => 0,
            })
            .set_lkey(self.key);

        let mut send_req = unsafe { Pin::new_unchecked(&mut send_req) };
        crate::rdma::payload::Payload::<ib_ud_wr>::finalize(send_req.as_mut());

        self.inner.post(&send_req.as_ref())
    }

    #[inline]
    fn get_pending_reqs(&self) -> usize { 
        self.inner.get_pending()
    }
}

impl<'a> super::super::RPCFactory for UDDatagram<'a> {
    type ConnMeta = (EndPoint, u32);
    type ConnType = UDSession<'a>;

    type ConnResult = crate::rdma::Err;

    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult> {
        let (endpoint, key) = meta;
        Ok(UDSession::<'a> {
            meta: endpoint,
            key: key,
            inner: self.clone(),
        })
    }
}

impl<UDFactory : crate::conn::MetaFactory> super::super::GenHyperMeta<UDFactory>
    for crate::datagram::ud_receiver::UDReceiver<'_>    
{
    type Args = (alloc::string::String, u64); // gid, service ID

    fn generate_hyper(&self, _args: &Self::Args) -> UDFactory::HyperMeta {
        unimplemented!();
    }
}
