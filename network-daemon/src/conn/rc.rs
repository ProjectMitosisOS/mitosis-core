use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::string::String;
use core::pin::Pin;

use crate::conn::{RDMAConn, IOResult, ConnErr, PathResult, ConnState, ConnTarget};

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
    state: ConnState,
}

/// server-side service to handle in-coming connections
pub struct RCService<'a> {
    rcontext: &'a RContext<'a>,
    rcontrol: Pin<Box<RCtrl<'a>>>,
    service_id: usize
}

impl<'a> RDMAConn for RCConn<'a> {
    /// connect to the target identified by conn_target
    fn conn(&mut self, conn_target: &ConnTarget) -> IOResult<()> {
        let inner_sa_client = crate::get_inner_sa_client();
        let mut explorer = IBExplorer::new();
        let _ = explorer.resolve(1, &self.rcontext, &String::from(conn_target.target_gid), inner_sa_client);
        let path_res = explorer.get_path_result();
        if path_res.is_none() {
            return Err(ConnErr::ERR_PATH_NOT_FOUND);
        }
        let path_res = path_res.unwrap();
        self.conn_path_result(conn_target, path_res)
    }

    /// connect_path_result will directly reuse the path result instead of resolve the target gid
    fn conn_path_result(&mut self, conn_target: &ConnTarget, path_res: PathResult) -> IOResult<()> {
        if self.is_connected() {
            return Err(ConnErr::ERR_CONN_STATE);
        }
        let mrc = unsafe { Arc::get_mut_unchecked(&mut self.rc) };
        let result = mrc.connect(conn_target.qd_hint, path_res, conn_target.remote_service_id as u64);
        match result {
            Ok(()) => {
                self.bring_to_connected();
                return Ok(());
            },
            Err(errno) => {
                use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module::bindings::{EAGAIN};
                match errno.to_kernel_errno() as u32 {
                    EAGAIN => {
                        return Err(ConnErr::ERR_TRY_AGAIN);
                    },
                    _ => {
                        return Err(ConnErr::ERR_OPERATION);
                    }
                }
            }
        };
    }

    fn get_conn_state(&self) -> ConnState {
        self.state
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

impl<'a> RCConn<'a> {
    pub fn new(target_gid: &'a str, ctx: &'a RContext<'a>) -> Option<Self> {
        let rc = RC::new(ctx, core::ptr::null_mut(), core::ptr::null_mut());
        if rc.is_none() {
            println!("unable to create RC qp");
            return None;
        }
        let rc = rc.unwrap();
        Some(Self {
                rc: rc,
                rcontext: ctx,
                state: ConnState::CREATED
            }
        ) 
    }
    
    fn is_connected(&self) -> bool {
        self.state == ConnState::CONNECTED
    }

    fn bring_to_connected(&mut self) {
        self.state = ConnState::CONNECTED;
    }
}

impl<'a> RCService<'a> {
    pub fn new(service_id: usize, ctx: &'a RContext<'a>) -> Option<Arc<Self>> {
        let ctrl = RCtrl::create(service_id, ctx).unwrap();
        Some(Arc::new(Self {
            rcontext: ctx,
            service_id: service_id,
            rcontrol: ctrl
        }))
    }
}
