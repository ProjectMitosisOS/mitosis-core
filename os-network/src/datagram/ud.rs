// TODO: should move ot the datagram folder
use alloc::sync::Arc;

use crate::future::{Async, Future, Poll};

use core::marker::PhantomData;
use core::option::Option;

use KRdmaKit::cm::SidrCM;
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

    pub fn clone(&self) -> Self {
        Self {
            ud: self.ud.clone(),
            phantom: PhantomData,
        }
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
                    return Err(Err::WCErr(wc.status.into()));
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
    // gid, service id, qd hint
    type HyperMeta = (alloc::string::String, u64, u64);

    // ud endpoint, local memory protection key
    type Meta = (KRdmaKit::cm::EndPoint, u32);

    type MetaResult = Err;

    fn create_meta(&self, meta: Self::HyperMeta) -> Result<Self::Meta, Self::MetaResult> {
        let (gid, service_id, qd_hint) = meta;
        let path_res = self.rctx.explore_path(gid, service_id).ok_or(Err::Other)?;
        let mut sidr_cm = SidrCM::new(&self.rctx, core::ptr::null_mut()).ok_or(Err::Other)?;
        let endpoint = sidr_cm
            .sidr_connect(path_res, service_id, qd_hint)
            .map_err(|_| Err::Other)?;
        Ok((endpoint, unsafe { self.rctx.get_lkey() }))
    }
}
