// TODO: should move ot the datagram folder
use alloc::sync::Arc;

use crate::future::{Async, Future, Poll};

use core::marker::PhantomData;
use core::option::Option;

use KRdmaKit::device::{RContext, RNIC};
use KRdmaKit::qp::UD;
use KRdmaKit::rust_kernel_rdma_base::*;

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
                    crate::log::debug!("check wc: {}", wc.status);
                    return Err(Err::Other);
                }
                Ok(Async::Ready(wc))
            }
            _ => Err(Err::Other),
        }
    }
}

impl crate::conn::Conn for UDDatagram<'_> {
    type ReqPayload = crate::rdma::payload::Payload<ib_ud_wr>;

    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult> {
        let mut bad_wr: *mut ib_send_wr = core::ptr::null_mut();
        let err = unsafe {
            bd_ib_post_send(
                self.ud.get_qp(),
                req.get_wr_ptr() as *mut _,
                &mut bad_wr as *mut _,
            )
        };
        if err != 0 {
            return Err(Err::Other);
        }

        Ok(())
    }
}

impl crate::conn::Factory for UDFactory<'_> {
    type ConnMeta = ();
    type ConnType<'a>
    where
        Self: 'a,
    = UDDatagram<'a>;

    type ConnResult = Err;

    fn create(&self, _meta: Self::ConnMeta) -> Result<Self::ConnType<'_>, Self::ConnResult> {
        let ud = UD::new(&self.rctx).ok_or(Err::Other)?;
        Ok(UDDatagram {
            ud: ud,
            phantom: PhantomData,
        })
    }
}

impl crate::conn::MetaFactory for UDFactory<'_> {
    type HyperMeta = ();
    type Meta = KRdmaKit::cm::EndPoint;
    type MetaResult = Err; 

    fn create(&self, meta: Self::HyperMeta) -> Result<Self::Meta, Self::MetaResult> { 
        unimplemented!(); 
    }

}
