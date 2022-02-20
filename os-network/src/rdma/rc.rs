use alloc::sync::Arc;

use core::option::Option;
use core::marker::PhantomData;

use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::device::{RContext, RNIC};
use KRdmaKit::qp::RC;
use KRdmaKit::qp::RCOp;
use rust_kernel_linux_util as log;

use super::payload::rc_payload::{RCReqPayload, RCCompPayload};
use super::{ConnErr, ConnMetaWPath};

pub struct RCFactory<'a> {
    rctx: RContext<'a>,
}

impl<'a> RCFactory<'a> {
    pub fn new(hca: &'a RNIC) -> Option<Self> {
        RContext::create(hca).map(|c| Self { rctx: c })
    }

    pub fn get_context(&self) -> &RContext<'_> {
        &self.rctx
    }
}

impl crate::ConnFactory for RCFactory<'_> {
    type ConnMeta = super::ConnMeta;
    type ConnType<'a>
	where Self: 'a
        = RCConn<'a>;
    type ConnResult = super::ConnErr;

    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType<'_>, ConnErr>
    {
        let path_res = self
            .rctx
            .explore_path(meta.gid, meta.service_id)
            .ok_or(ConnErr::PathNotFound)?;

        let mut rc = RC::new(&self.rctx, core::ptr::null_mut(), core::ptr::null_mut())
            .ok_or(ConnErr::CreateQPErr)?;

        // connect the RC
        let mrc = unsafe { Arc::get_mut_unchecked(&mut rc) };
        match mrc.connect(meta.qd_hint, path_res, meta.service_id as u64) {
            Ok(_) => Ok(RCConn::<'_> { rc: rc, phantom : PhantomData } ),
            Err(_) => Err(ConnErr::ConnErr),
        }
    }
}

pub struct RCFactoryWPath<'a> {
    rctx: RContext<'a>,
}

impl<'a> RCFactoryWPath<'a> {
    pub fn new(hca: &'a RNIC) -> Option<Self> {
        RContext::create(hca).map(|c| Self { rctx: c })
    }

    pub fn convert_meta(&self, meta: super::ConnMeta) -> Option<super::ConnMetaWPath> { 
        ConnMetaWPath::new(&self.rctx, meta) 
    }
}

impl crate::ConnFactory for RCFactoryWPath<'_> {
    type ConnMeta = super::ConnMetaWPath;
    type ConnType<'a>
	where Self: 'a
        = RCConn<'a>;
    type ConnResult = super::ConnErr;

    // Note: the path_res in the meta is recommended to be generated via the context of RCFactoryWPath.rctx
    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType<'_>, super::ConnErr>
    {
        let mut rc = RC::new(&self.rctx, core::ptr::null_mut(), core::ptr::null_mut())
            .ok_or(ConnErr::CreateQPErr)?;

        // connect the RC
        let mrc = unsafe { Arc::get_mut_unchecked(&mut rc) };
        match mrc.connect(meta.qd_hint, meta.path, meta.service_id as u64) {       
            Ok(_) => Ok(RCConn::<'_> { rc: rc, phantom : PhantomData } ),
            Err(_) => Err(ConnErr::ConnErr), // TODO: need to filter against the connection results
        }
    }
}


// Connection 

pub struct RCConn<'a> {
    rc: Arc<RC>,
    phantom : PhantomData<&'a ()>,
}

impl RCConn<'_> { 
    pub fn get_status(&mut self) -> Option<super::QPStatus> { 
        self.rc.get_status().map(|status| { 
            match status { 
                ib_qp_state::IB_QPS_RTS => super::QPStatus::RTS, 
                ib_qp_state::IB_QPS_RTR => super::QPStatus::RTR,
                _ => super::QPStatus::Other
            }
        })
    }
}

impl crate::Conn for RCConn<'_> {

    type ReqPayload = RCReqPayload;
    type CompPayload = RCCompPayload;
    type IOResult = super::Err; 

    fn post(&mut self, req: &Self::ReqPayload) -> Result<(),Self::IOResult> {
        let mut op = RCOp::new(&self.rc);
        op.post_send(req.get_sge(), req.get_wr()).map_err(|_x| {
            // TODO: need to be refined according to the error number
            super::Err::Other
        })
    }

    fn poll(&mut self) -> Result<Self::CompPayload,Self::IOResult> {
        let mut op = RCOp::new(&self.rc);
        let mut wc: ib_wc = Default::default();
        let result = op.pop_with_wc(&mut wc as *mut ib_wc);
        if result.is_none() {
            return Err(super::Err::Retry);
        }
        let result = result.unwrap();
        let result = unsafe {
            (*result).status
        };
        if result != ib_wc_status::IB_WC_SUCCESS {
            log::error!("poll cq with err: {}", result);
            return Err(super::Err::Other)
        } else {
            return Ok(RCCompPayload{});
        }
    }

    fn wait_til_comp(&mut self) -> Result<Self::CompPayload,Self::IOResult> {
        loop {
            let result = self.poll();
            if result.is_ok() {
                return result;
            }
            let err = result.err().unwrap();
            if err == super::Err::Retry {
                continue;
            }
            return Err(err);
        }
    }
}
