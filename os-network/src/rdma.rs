use alloc::string::String; 

pub struct ConnMeta {
    pub target_gid: String,
    pub remote_service_id: u64,
    pub qd_hint: u64
}

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

pub type ConnResult<T> = Result<T, ConnErr>; 

#[derive(Debug)]
pub enum Err {
    // TODO: Need to be refined, should be more detailed

    /// Timeout error
    Timeout = 0,

    /// Other general error
    Other, 
}

pub type IOResult<T> = Result<T, Err>;

pub mod rc;

