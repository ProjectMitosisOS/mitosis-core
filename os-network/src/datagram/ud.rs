// TODO: should move ot the datagram folder
use alloc::sync::Arc;

use crate::bytes::BytesMut;
use crate::future::{Async, Future, Poll};

use core::marker::PhantomData;
use core::option::Option;

use KRdmaKit::device::{RContext, RNIC};
use KRdmaKit::mem::RMemPhy;
use KRdmaKit::qp::{UDOp, UD};
use KRdmaKit::rust_kernel_rdma_base::*;

use super::msg::UDMsg;
use super::{Datagram, Factory};

// XD: should tuned. maybe use it as a global configure?
pub const MAX_MTU: usize = 4096;
// 40: global routing header (GRH)
pub const MAX_MSG_SZ: usize = MAX_MTU - 40;

pub struct UDFactory<'a> {
    rctx: RContext<'a>,
}

impl<'a> UDFactory<'a> {
    pub fn new(hca: &'a RNIC) -> Option<Self> {
        RContext::create(hca).map(|c| Self { rctx: c })
    }

    pub fn get_context(&self) -> &RContext<'_> {
        &self.rctx
    }
}

pub struct UDDatagram<'a> {
    pub(crate) ud: Arc<UD>,
    phantom: PhantomData<&'a ()>,
}

impl UDDatagram<'_> {
    pub fn get_qp(&self) -> Arc<UD> {
        self.ud.clone()
    }
}

use crate::rdma::Err;

impl Future for UDDatagram<'_> {
    type Output = ib_wc;
    type Error = Err;

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        let mut wc: ib_wc = Default::default();
        match unsafe { bd_ib_poll_cq(self.ud.get_cq(), 1, &mut wc) } {
            0 => Ok(Async::NotReady),
            1 => {
                // FIXME: should dispatch according to the wc_status
                if wc.status != ib_wc_status::IB_WC_SUCCESS {
                    return Err(Err::Other);
                }
                Ok(Async::Ready(wc))
            }
            _ => Err(Err::Other),
        }
    }
}

impl crate::Datagram for UDDatagram<'_> {
    type AddressHandler =  KRdmaKit::cm::EndPoint;
    type Msg = UDMsg;
    type Key = u32;

    fn post_msg(
        &mut self,
        addr: &Self::AddressHandler,
        msg: &Self::Msg,
        key: &Self::Key,
    ) -> Result<(), Self::IOResult> {
        crate::log::debug!("check posted msg: {:?}", addr); 
        let mut op = UDOp::new(&self.ud);
        let res = op.send(msg.get_pa(), *key, &addr, msg.get_bytes().len());
        if res.is_err() {
            return Err(Err::Other);
        }
        Ok(())
    }
}

impl Factory for UDFactory<'_> {
    type CreateMeta = ();
    type DatagramType<'a>
    where
        Self: 'a,
    = UDDatagram<'a>;
    type CreateResult = Err;

    fn create(&self, meta: Self::CreateMeta) -> Result<Self::DatagramType<'_>, Self::CreateResult> {
        let ud = UD::new(&self.rctx).ok_or(Err::Other)?;
        Ok(UDDatagram {
            ud: ud,
            phantom: PhantomData,
        })
    }
}
