#[allow(non_snake_case)]

use alloc::string::String;

use KRdmaKit::device::RContext;
use KRdmaKit::rust_kernel_rdma_base::sa_path_rec;

pub struct ConnMeta {
    pub gid: String,
    pub service_id: u64,
    pub qd_hint: u64,
}

#[derive(Clone, Copy)]
pub struct ConnMetaWPath {
    pub path: sa_path_rec,
    pub service_id: u64,
    pub qd_hint: u64,
}

impl ConnMetaWPath {
    pub fn new(ctx: &RContext, meta: ConnMeta) -> core::option::Option<Self> {
        ctx.explore_path(meta.gid.clone(), meta.service_id)
            .map(|path| Self {
                path: path,
                service_id: meta.service_id,
                qd_hint: meta.qd_hint,
            })
    }
}

#[derive(Debug)]
pub enum ConnErr {
    PathNotFound = 0,

    CreateQPErr,

    Timeout,

    QPNotReady,

    ConnErr,
}

#[derive(Debug, PartialEq)]
pub enum Err {
    // TODO: Need to be refined, should be more detailed
    /// Timeout error
    Timeout = 0,

    /// Retry, used to indicate retrying the erroneous function call, typically `ib_poll_cq`
    Retry,

    /// Other general error
    Other,
}

#[derive(Debug)]
pub enum QPStatus { 
    RTR,
    RTS, 
    Other, 
}

pub mod rc;
pub mod payload;
// pub mod ud;
pub mod dc; 
