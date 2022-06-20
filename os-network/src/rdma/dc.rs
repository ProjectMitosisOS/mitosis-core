use alloc::sync::Arc;

use KRdmaKit::device::RContext;
use KRdmaKit::qp::{DCOp, DC, DCTServer,Config};
use KRdmaKit::rust_kernel_rdma_base::ib_dc_wr;
use KRdmaKit::rust_kernel_rdma_base::*;

use rust_kernel_linux_util as log;

use core::marker::PhantomData;

pub type DCTarget = DCTServer;

pub struct DCFactory<'a> {
    rctx: &'a RContext<'a>,
}

impl<'a> DCFactory<'a> {
    pub fn new(ctx: &'a RContext<'a>) -> Self {
        Self { rctx: ctx }
    }

    pub fn get_context(&self) -> &RContext<'_> {
        self.rctx
    }

    pub fn create_target(&self, key : u64) -> core::option::Option<Arc<DCTarget>> { 
        let mut config : Config = Default::default();
        config.dc_key = key as _;
        DCTarget::new_from_config(config, self.rctx)
    }
}

pub struct DCConn<'a> {
    dc: Arc<DC>,
    phantom: PhantomData<&'a ()>,
    watermark : u64,
}

use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;

impl crate::Conn for DCConn<'_> {
    type ReqPayload = super::payload::Payload<ib_dc_wr>;
    type CompPayload = ();
    type IOResult = super::Err;

    #[inline]
    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult> {
        compiler_fence(SeqCst);
        let mut op = DCOp::new(&self.dc);
        unsafe {
            op.post_send_raw(req.get_wr_ptr() as *mut _)
                .map_err(|_x| super::Err::Other)
        }
    }
}

use crate::future::{Async, Future, Poll};

impl Future for DCConn<'_> {
    type Output = ib_wc;
    type Error = super::Err;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        compiler_fence(SeqCst);
        let mut wc: ib_wc = Default::default();
        let cq = self.dc.get_cq();
        let ret = unsafe { bd_ib_poll_cq(cq, 1, &mut wc) };
        match ret {
            0 => {
                return Ok(Async::NotReady);
            }
            1 => {
                if wc.status != ib_wc_status::IB_WC_SUCCESS {
                    return Err(super::Err::WCErr(wc.status.into()));
                } else {
                    return Ok(Async::Ready(wc));
                }
            }
            _ => {
                log::error!("ib_poll_cq returns {}", ret);
                return Err(super::Err::Other);
            }
        }
    }
}

impl<'a> Clone for DCConn<'a> {
    /// Clone is fine for DCQP
    /// This is because
    /// 1. it is behind ARC
    /// 2. the raw QP is thread-safe
    fn clone(&self) -> Self {
        Self { 
            dc : self.dc.clone(), 
            phantom: PhantomData,    
            watermark : 0,
        }
    }
}

impl crate::conn::Factory for DCFactory<'_> {
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
                watermark : 0,
            }),
            None => Err(super::ConnErr::CreateQPErr),
        }
    }
}

