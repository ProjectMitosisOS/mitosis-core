#[allow(non_snake_case)]

use alloc::string::String;

use KRdmaKit::device::RContext;
use KRdmaKit::rust_kernel_rdma_base::sa_path_rec;

pub struct ConnMeta {
    pub gid: String,
    pub service_id: u64,
    pub qd_hint: u64,
}

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

pub type ConnResult<T> = Result<T, ConnErr>;

#[derive(Debug)]
pub enum Err {
    // TODO: Need to be refined, should be more detailed
    /// Timeout error
    Timeout = 0,

    /// Other general error
    Other,
}

#[derive(Debug)]
pub enum QPStatus { 
    RTR,
    RTS, 
    Other, 
}

pub type IOResult<T> = Result<T, Err>;

pub mod rc;
// pub mod ud;
// pub mod dc; 
