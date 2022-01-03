use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use core::pin::Pin;

use crate::conn::{ConnErr, ConnTarget, IOResult, PathResult, RDMAConn};

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::device::RContext;
use KRdmaKit::ib_path_explorer::IBExplorer;
use KRdmaKit::qp::RC;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module::{println, KernelResult};
use KRdmaKit::SAClient;

/// client-side connection
pub struct RCConn<'a> {
    rc: Arc<RC>,
    rcontext: &'a RContext<'a>,
}

// control-path operations
impl<'a> RCConn<'a> {

    pub fn create(target: &ConnTarget, ctx: &'a RContext<'a>) -> IOResult<Self> {
        // first establish path 
        let inner_sa_client = unsafe { crate::get_inner_sa_client() };
        let mut explorer = IBExplorer::new();
        let _ = explorer.resolve(
            1,
            ctx,
            &String::from(target.target_gid),
            inner_sa_client,
        );
        let path_res = explorer.get_path_result().ok_or(ConnErr::PATH_NOT_FOUND)?;
        Self::create_w_path(target, path_res, ctx) 
    }

    pub fn create_w_path(target: &ConnTarget,path_res: PathResult, ctx: &'a RContext<'a>) -> IOResult<Self> {
        let mut rc = RC::new(ctx, core::ptr::null_mut(), core::ptr::null_mut()).
            ok_or(ConnErr::OPERATION)?;

        // connect the RC 
        let mrc = unsafe { Arc::get_mut_unchecked(&mut rc) }; 
        match mrc.connect(
            target.qd_hint,
            path_res,
            target.remote_service_id as u64,
        ) {
            Ok(_) => Ok(Self { rc : rc, rcontext : ctx}), 
            Err(_) => Err(ConnErr::CONNErr) // XD: TODO, should distinguish
        }
    }    
}

impl<'a> RDMAConn for RCConn<'a> {
    fn ready(&self) -> IOResult<()> { 
        // XD: TODO: how to implement it without storing a conn state? 
        unimplemented!()
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

/// server-side service to handle in-coming connections
pub struct RCService<'a> {
    rcontext: &'a RContext<'a>,
    rcontrol: Pin<Box<RCtrl<'a>>>,
    service_id: usize,
}

impl<'a> RCService<'a> {
    pub fn new(service_id: usize, ctx: &'a RContext<'a>) -> Option<Arc<Self>> {
        let ctrl = RCtrl::create(service_id, ctx).unwrap();
        Some(Arc::new(Self {
            rcontext: ctx,
            service_id: service_id,
            rcontrol: ctrl,
        }))
    }
}
