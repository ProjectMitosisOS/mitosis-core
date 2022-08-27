use KRdmaKit::context::Context;
use KRdmaKit::DatapathError;
#[allow(non_snake_case)]
use alloc::string::String;
use alloc::sync::Arc;

pub const MAX_GID_LEN: usize = 40; // The maximum string length of IPv6

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawGID {
    inner: [u8; MAX_GID_LEN],
    real_len: usize,
}

impl Default for RawGID {
    fn default() -> Self {
        Self {
            inner: [0; MAX_GID_LEN],
            real_len: 0,
        }
    }
}

impl RawGID {
    pub fn new(gid: alloc::string::String) -> core::option::Option<Self> {
        // a loose check
        if gid.len() <= MAX_GID_LEN {
            let mut res: [u8; MAX_GID_LEN] = [0; MAX_GID_LEN];
            res[..gid.len()].copy_from_slice(gid.as_bytes());
            return Some(Self {
                inner: res,
                real_len: gid.len(),
            });
        }
        None
    }

    pub fn len(&self) -> usize {
        self.real_len
    }
}

impl alloc::string::ToString for RawGID {
    fn to_string(&self) -> String {
        core::str::from_utf8(&self.inner[0..self.real_len])
            .unwrap()
            .to_string()
    }
}

pub struct ConnMeta {
    pub gid: KRdmaKit::rdma_shim::bindings::ib_gid,
    pub service_id: u64,
    pub port: u8,
}

pub struct DCCreationMeta {
    pub port: u8,
}

#[derive(Debug)]
pub enum Err {
    /// data path error
    ///
    DatapathError(DatapathError),
    /// rdma work completion error
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
    IB_WC_SUCCESS = 0,
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
    fn from(err: u32) -> Self {
        unsafe { core::mem::transmute(err) }
    }
}

pub mod dc;
pub mod payload;
pub mod rc;
