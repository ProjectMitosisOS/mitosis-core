pub mod rc;
pub mod dc;
pub mod ud;

pub type PathResult = KRdmaKit::rust_kernel_rdma_base::sa_path_rec;

#[derive(Debug)]
pub enum IOErr {
    ConnErr(ConnErr),
    RDMAErr(RDMAErr),
    /// Other operation error will be catagorized as `Other`
    /// E.g.: Error in get qp status
    Other
}

/// RDMA control path operation related error
#[derive(Debug)]
pub enum ConnErr {
    /// Error in finding the path whiling connecting
    /// with raw gid
    PathNotFound = 0,

    /// Timeout error
    Timeout,

    /// Error when the qp is not ready a.k.a. RTS state
    QPNotReady,

    /// General error in the rdma connection operation
    ConnErr,
}

/// RDMA data path operation related error
#[derive(Debug)]
pub enum RDMAErr {
    // TODO: Need to be refined, should be more detailed

    /// Timeout error
    Timeout = 0,

    /// Other general error
    RDMAErr
}

pub type IOResult<T> = Result<T, IOErr>;
/// ConnTarget contains necessary information to identify a remote rdma nic's service (rctrl)
/// XD: TODO: describe the following fields 
pub struct ConnTarget<'a> {
    pub target_gid: &'a str,
    pub remote_service_id: u64,
    pub qd_hint: u64
}

/// RDMAConn is the abstract network connections of mitosis
pub trait RDMAConn {
    // control path
//    fn conn(&mut self, conn_target: &ConnTarget) -> IOResult<()>;
//    fn conn_path_result(&mut self, conn_target: &ConnTarget, path_res: PathResult) -> IOResult<()>;
//    fn get_conn_state(&self) -> ConnState;
    fn ready(&self) -> IOResult<()>; 

    // data path
    fn one_sided_read(&self, local_addr: u64, remote_addr: u64) -> IOResult<()>;
    fn one_sided_write(&self, local_addr: u64, remote_addr: u64) -> IOResult<()>;
    fn send_msg(&self, local_addr: u64) -> IOResult<()>;
    fn recv_msg(&self, local_addr: u64) -> IOResult<()>;
}
