use alloc::boxed::Box;
use alloc::sync::Arc;
use core::pin::Pin;

use crate::conn::{IOErr, ConnErr};
use crate::conn::{ConnTarget, IOResult, PathResult, RDMAConn};
use crate::conn::get_path_result;

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::device::RContext;
use KRdmaKit::qp::RC;
use KRdmaKit::rust_kernel_rdma_base::ib_qp_state;

/// client-side connection
pub struct RCConn<'a> {
    rc: Arc<RC>,
    rcontext: &'a RContext<'a>,
}

// control-path operations
impl<'a> RCConn<'a> {

    pub fn create(target: &ConnTarget, ctx: &'a RContext<'a>) -> IOResult<Self> {
        let path_res = get_path_result(target.target_gid, ctx)?;
        Self::create_w_path(target, path_res, ctx) 
    }

    pub fn create_w_path(target: &ConnTarget,path_res: PathResult, ctx: &'a RContext<'a>) -> IOResult<Self> {
        let mut rc = RC::new(ctx, core::ptr::null_mut(), core::ptr::null_mut()).
            ok_or(IOErr::Other)?;

        // connect the RC 
        let mrc = unsafe { Arc::get_mut_unchecked(&mut rc) }; 
        match mrc.connect(
            target.qd_hint,
            path_res,
            target.remote_service_id as u64,
        ) {
            Ok(_) => Ok(Self { rc : rc, rcontext : ctx}), 
            Err(_) => Err(IOErr::ConnErr(ConnErr::ConnErr)) // XD: TODO, should distinguish
        }
    }    
}

// data-path operation
impl<'a> RDMAConn for RCConn<'a> {
    fn ready(&self) -> IOResult<()> { 
        let status = self.rc.get_status().ok_or(IOErr::Other)?;
        if status != ib_qp_state::IB_QPS_RTS {
            Err(IOErr::ConnErr(ConnErr::QPNotReady))
        } else {
            Ok(())
        }
    }

    fn one_sided_read(&self, local_addr: u64, remote_addr: u64) -> IOResult<()> {
        unimplemented!()
    }

    fn one_sided_write(&self, local_addr: u64, remote_addr: u64) -> IOResult<()> {
        unimplemented!()
    }

    fn send_msg(&self, local_addr: u64) -> IOResult<()> {
        unimplemented!()
    }

    fn recv_msg(&self, local_addr: u64) -> IOResult<()> {
        unimplemented!()
    }
}

/// server-side service to handle in-coming connections, a simple wrapper over RCtrl
pub struct RCService<'a> {
    rcontrol: Pin<Box<RCtrl<'a>>>,
}

impl<'a> RCService<'a> {
    pub fn new(service_id: usize, ctx: &'a RContext<'a>) -> Option<Arc<Self>> {
        let ctrl = RCtrl::create(service_id, ctx)?;
        Some(Arc::new(Self {
            rcontrol: ctrl,
        }))
    }
}
