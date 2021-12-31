pub mod rc;
pub mod dc;
pub mod ud;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module::KernelResult;

pub type PathResult = KRdmaKit::rust_kernel_rdma_base::sa_path_rec;

pub enum ConnErr {
    ERR_OPERATION = 0,
    ERR_PATH_NOT_FOUND,
    ERR_TIMEOUT,
    ERR_TRY_AGAIN,
    ERR_CONN_STATE,
}

pub type IOResult<T> = Result<T, ConnErr>;

#[derive(PartialEq, Copy, Clone)]
pub enum ConnState {
    CREATED,
    CONNECTED
}

/// ConnTarget contains necessary information to identify a remote rdma nic's service (rctrl)
pub struct ConnTarget<'a> {
    pub target_gid: &'a str,
    pub remote_service_id: u64,
    pub qd_hint: u64
}

/// RDMAConn is the abstract network connections of mitosis
pub trait RDMAConn {
    // control path
    fn conn(&mut self, conn_target: &ConnTarget) -> IOResult<()>;
    fn conn_path_result(&mut self, conn_target: &ConnTarget, path_res: PathResult) -> IOResult<()>;
    fn get_conn_state(&self) -> ConnState;

    // data path
    fn one_sided_read(&self, local_addr: u64, remote_addr: u64) -> IOResult<()>;
    fn one_sided_write(&self, local_addr: u64, remote_addr: u64) -> IOResult<()>;
    fn send_msg(&self, local_addr: u64) -> IOResult<()>;
    fn recv_msg(&self, local_addr: u64) -> IOResult<()>;
}
