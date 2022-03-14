use alloc::sync::Arc;

use KRdmaKit::device::RContext;
use KRdmaKit::device::RNIC;
use KRdmaKit::qp::{DC, DCOp};
use KRdmaKit::rust_kernel_rdma_base::ib_dc_wr;
use KRdmaKit::rust_kernel_rdma_base::*;

use rust_kernel_linux_util as log;

use core::marker::PhantomData;

pub struct DCFactory<'a> {
    rctx: RContext<'a>,
}

impl<'a> DCFactory<'a> {
    pub fn new(hca: &'a RNIC) -> Option<Self> {
        RContext::create(hca).map(|c| Self { rctx: c })
    }

    pub fn get_context(&self) -> &RContext<'_> {
        &self.rctx
    }
}

pub struct DCConn<'a> {
    dc: Arc<DC>,
    phantom: PhantomData<&'a ()>,
}

impl crate::Conn for DCConn<'_> {
    type ReqPayload = super::payload::Payload<ib_dc_wr>;
    type CompPayload = ();
    type IOResult = super::Err;

    fn post(&mut self, req: &Self::ReqPayload) -> Result<(),Self::IOResult> {
        let mut op = DCOp::new(&self.dc);
        unsafe {
            op.post_send_raw(req.get_wr_ptr() as *mut _).map_err(|_x| {
                super::Err::Other
            })
        }
    }
}

impl crate::ConnFactory for DCFactory<'_> {
    type ConnMeta = ();
    type ConnType<'a>
    where
        Self: 'a,
    = DCConn<'a>;
    type ConnResult = super::ConnErr;

    fn create(&self, _meta: Self::ConnMeta) -> Result<Self::ConnType<'_>, Self::ConnResult> {
        let dc = DC::new(&self.rctx);
        match dc {
            Some(dc) => Ok(DCConn::<'_> {
                dc: dc,
                phantom: PhantomData,
            }),
            None => Err(super::ConnErr::CreateQPErr),
        }
    }
}

use crate::future::{Async,Future,Poll};

impl Future for DCConn<'_> {
    type Output = ib_wc;
    type Error = super::Err;

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        let mut wc: ib_wc = Default::default();
        let cq = self.dc.get_cq();
        let ret = unsafe {
            bd_ib_poll_cq(cq, 1, &mut wc)
        };
        match ret {
            0 => {
                return Ok(Async::NotReady);
            },
            1 => {
                if wc.status != ib_wc_status::IB_WC_SUCCESS {
                    log::error!("poll cq with err: {}", wc.status);
                    return Err(super::Err::Other);
                } else {
                    return Ok(Async::Ready(wc));
                }
            },
            _ => {
                log::error!("ib_poll_cq returns {}", ret);
                return Err(super::Err::Other);
            },
        }
    }
}