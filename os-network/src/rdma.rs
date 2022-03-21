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
    Timeout,

    /// Retry, used to indicate retrying the erroneous function call, typically `ib_poll_cq`
    Retry,

    Empty, 
    /// Other general error
    ///     
    Other,
    /// ib_wc error
    /// 
    WCErr(WCStatus),
}

impl Err {
    pub fn is_wc_err(&self) -> bool {
        match self {
            Self::WCErr(_) => true,
            _ => false,
        }
    }

    pub fn unwrap_wc_err(self) -> WCStatus {
        match self {
            Self::WCErr(status) => status,
            _ => panic!("called `unwrap_wc_err()` on a non-wc_err value"),
        }
    }
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum WCStatus {
    IB_WC_SUCCESS,
	IB_WC_LOC_LEN_ERR,
	IB_WC_LOC_QP_OP_ERR,
	IB_WC_LOC_EEC_OP_ERR,
	IB_WC_LOC_PROT_ERR,
	IB_WC_WR_FLUSH_ERR,
	IB_WC_MW_BIND_ERR,
	IB_WC_BAD_RESP_ERR,
	IB_WC_LOC_ACCESS_ERR,
	IB_WC_REM_INV_REQ_ERR,
	IB_WC_REM_ACCESS_ERR,
	IB_WC_REM_OP_ERR,
	IB_WC_RETRY_EXC_ERR,
	IB_WC_RNR_RETRY_EXC_ERR,
	IB_WC_LOC_RDD_VIOL_ERR,
	IB_WC_REM_INV_RD_REQ_ERR,
	IB_WC_REM_ABORT_ERR,
	IB_WC_INV_EECN_ERR,
	IB_WC_INV_EEC_STATE_ERR,
	IB_WC_FATAL_ERR,
	IB_WC_RESP_TIMEOUT_ERR,
	IB_WC_GENERAL_ERR,
}

impl From<u32> for WCStatus {
    fn from(errno: u32) -> Self {
        unsafe {
            core::mem::transmute(errno)
        }
    }
}

#[derive(Debug)]
pub enum QPStatus { 
    RTR,
    RTS, 
    Other, 
}

pub mod payload;
pub mod rc;
pub mod dc; 
