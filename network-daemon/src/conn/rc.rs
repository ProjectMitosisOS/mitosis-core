use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::string::String;
use core::pin::Pin;

use crate::conn::RDMAConn;
use crate::sa_client;

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::qp::RC;
use KRdmaKit::device::RContext;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module::{KernelResult, println};
use KRdmaKit::SAClient;
use KRdmaKit::ib_path_explorer::IBExplorer;

/// client-side connection 
pub struct RCConn<'a> {
    rc: Arc<RC>,
    rcontext: &'a RContext<'a>,
    target_gid: &'a str,
    remote_service_id: u64,
    qd_hint: u64
}

/// server-side service to handle in-coming connections
pub struct RCService<'a> {
    rcontext: &'a RContext<'a>,
    rcontrol: Pin<Box<RCtrl<'a>>>,
    service_id: usize
}

impl<'a> RDMAConn for RCConn<'a> {
    /// connect to the target_gid, addr is not used in RCConn
    fn connect(&mut self, addr: &str) -> KernelResult<()> {
        let inner_sa_client = crate::get_inner_sa_client();
        let mut explorer = IBExplorer::new();
        let _ = explorer.resolve(1, &self.rcontext, &String::from(self.target_gid), inner_sa_client);
        let path_res = explorer.get_path_result().unwrap();
        let mrc = unsafe { Arc::get_mut_unchecked(&mut self.rc) };
        let result = mrc.connect(self.qd_hint, path_res, self.remote_service_id as u64);
        result
    }

    fn one_sided_read(&self, local_addr: u64, remote_addr: u64) -> KernelResult<()> {
        unimplemented!()
    }

    fn one_sided_write(&self, local_addr: u64, remote_addr: u64) -> KernelResult<()> {
        unimplemented!()
    }

    fn send_msg(&self, local_addr: u64) -> KernelResult<()> {
        unimplemented!()
    }

    fn recv_msg(&self, local_addr: u64) -> KernelResult<()> {
        unimplemented!()
    }
}

impl<'a> RCConn<'a> {
    pub fn new(target_gid: &'a str, ctx: &'a RContext<'a>, remote_service_id: u64, qd_hint: u64) -> Option<Self> {
        let rc = RC::new(&ctx, core::ptr::null_mut(), core::ptr::null_mut());
        if rc.is_none() {
            println!("unable to create RC qp");
            return None;
        }
        let rc = rc.unwrap();
        Some(Self {
                rc: rc,
                target_gid: target_gid,
                rcontext: ctx,
                remote_service_id: remote_service_id,
                qd_hint: qd_hint
            }
        )
        
    }
}

impl<'a> RCService<'a> {
    pub fn new(service_id: usize, ctx: &'a RContext<'a>) -> Option<Arc<Self>> {
        let ctrl = RCtrl::create(service_id, &ctx).unwrap();
        Some(Arc::new(Self {
            rcontext: &ctx,
            service_id: service_id,
            rcontrol: ctrl
        }))
    }
}
