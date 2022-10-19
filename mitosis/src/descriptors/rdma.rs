use crate::kern_wrappers::mm::PhyAddrType;
use os_network::rdma::dc::DCTarget;

use rust_kernel_rdma_base::bindings::ib_gid;

#[allow(dead_code)]
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct RDMADescriptor {
    // TODO: these fields are not correct.
    // The design of MITOSIS requires client using no-roundtrip to fork from the parent.
    // However, CM still uses one RTT
    pub service_id: u64,

    /// fields for ah access field
    pub port_num: u8,
    pub gid: ib_gid,
    pub lid: u16,

    /// fields for DCT accesses
    pub rkey: u32,
    pub dct_key: usize,
    pub dct_num: u32,

    pub mac_id : usize, 
}

use alloc::sync::Arc;

impl RDMADescriptor {
    pub fn new_from_dc_target_pool() -> core::option::Option<(Arc<DCTarget>, Self)> {
        let service = unsafe { crate::get_dc_target_service_mut() };
        let dc_target_meta = service
            .pop_one()
            .expect("failed to create a DCTarget from the pool");
        let dc_target = dc_target_meta.target.clone();

        // now fill the fields
        let my = Self {
            service_id: 0, // deprecated field
            port_num: dc_target.port_num(),
            gid: dc_target_meta.meta.gid,
            lid: dc_target_meta.meta.lid,

            rkey: dc_target_meta.rkey,
            dct_key: dc_target.dc_key() as usize,
            dct_num: dc_target.dct_num(),

            mac_id : unsafe { *crate::mac_id::get_ref() }
        };

        Some((dc_target, my))
    }

    pub fn get_rkey(&self) -> u32 {
        self.rkey
    }

    pub fn get_service_id(&self) -> u64 {
        self.service_id
    }

    pub fn set_rkey(&mut self, key: u32) -> &mut Self {
        self.rkey = key;
        self
    }

    pub fn set_service_id(&mut self, id: u64) -> &mut Self {
        self.service_id = id;
        self
    }
}

impl os_network::serialize::Serialize for RDMADescriptor {}

#[allow(dead_code)]
#[derive(Default)]
pub struct ReadMeta {
    pub addr: PhyAddrType,
    pub length: u64,
}

impl os_network::serialize::Serialize for ReadMeta {}
