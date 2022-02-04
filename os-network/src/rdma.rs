// TODO

use alloc::string::String; 

pub struct ConnMeta {
    pub target_gid: String,
    pub remote_service_id: u64,
    pub qd_hint: u64
}

pub mod rc;
