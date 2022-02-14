use alloc::sync::Arc;

use core::option::Option;

use KRdmaKit::rust_kernel_rdma_base::ib_qp_state;

use KRdmaKit::device::{RContext, RNIC};
use KRdmaKit::qp::RC;

use super::{ConnErr, ConnMetaWPath};

pub struct RCFactory<'a> {
    rctx: RContext<'a>,
}

impl<'a> RCFactory<'a> {
    pub fn new(hca: &'a RNIC) -> Option<Self> {
        RContext::create(hca).map(|c| Self { rctx: c })
    }
}

impl crate::ConnFactory for RCFactory<'_> {
    type ConnMeta = super::ConnMeta;
    type ConnType = RCConn;
    type ConnResult = super::ConnErr;

    fn create(&mut self, meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult>
    where
        Self::ConnType: crate::Conn,
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
            Ok(_) => Ok(RCConn { rc: rc }),
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

    pub fn convert_meta(&mut self, meta : super::ConnMeta) -> Option<super::ConnMetaWPath> { 
        ConnMetaWPath::new(&mut self.rctx, meta) 
    }
}

impl crate::ConnFactory for RCFactoryWPath<'_> {
    type ConnMeta = super::ConnMetaWPath;
    type ConnType = RCConn;
    type ConnResult = super::ConnErr;

    // Note: the path_res in the meta is recommended to be generated via the context of RCFactoryWPath.rctx
    fn create(&mut self, meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult>
    where
        Self::ConnType: crate::Conn,
    {
        let mut rc = RC::new(&self.rctx, core::ptr::null_mut(), core::ptr::null_mut())
            .ok_or(ConnErr::CreateQPErr)?;

        // connect the RC
        let mrc = unsafe { Arc::get_mut_unchecked(&mut rc) };
        match mrc.connect(meta.qd_hint, meta.path, meta.service_id as u64) {
            Ok(_) => Ok(RCConn { rc: rc }),
            Err(_) => Err(ConnErr::ConnErr),
        }
    }
}


pub struct RCConn {
    rc: Arc<RC>,
}

impl RCConn { 
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

impl crate::Conn for RCConn {
    type IOResult = super::Err;
    type ReqPayload = u64;
    type CompPayload = u64;

    fn post(&mut self, req: &Self::ReqPayload) -> Result<(),Self::IOResult> {
        unimplemented!();
    }

    fn poll(&mut self) -> Result<Self::CompPayload,Self::IOResult> {
        unimplemented!();
    }
}
