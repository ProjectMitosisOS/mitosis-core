use alloc::vec::Vec;

use os_network::rdma::dc::*;
use os_network::rdma::RawGID;

pub struct DCPool<'a> {
    pool: Vec<DCConn<'a>>,
    nic_idxs: Vec<usize>,
    // TODO: should initialize a DC Target pool
    // a simple DC key cannot protect all the stuffs in a fine-grained way
}

impl<'a> DCPool<'a> {
    pub fn get_dc_qp(&mut self, idx: usize) -> core::option::Option<&mut DCConn<'a>> {
        self.pool.get_mut(idx)
    }

    pub fn get_rdma_context(&self, pool_id : usize) -> core::option::Option<crate::descriptors::RDMADescriptor> {
        let nic_idx = self.nic_idxs.get(pool_id)?;

        let context = unsafe {crate::get_rdma_context_ref(*nic_idx) }.unwrap();

        Some(
        crate::descriptors::RDMADescriptor {
            gid: RawGID::new(context.get_gid_as_string()).unwrap(),
            service_id: crate::rdma_context::SERVICE_ID_BASE + *nic_idx as u64,
            rkey: unsafe { context.get_rkey() },
        })
    }
}

use os_network::Factory;

impl<'a> DCPool<'a> {
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        let mut res = Vec::new();
        let mut nic_idxs = Vec::new();

        for i in 0..config.max_core_cnt {
            let nic_idx = i % config.num_nics_used;
            res.push(
                unsafe { crate::get_dc_factory_ref(nic_idx) }
                    .expect("fatal, should not fail to create dc factory")
                    .create(())
                    .expect("Failed to create DC QP"),
            );
            nic_idxs.push(nic_idx);
        }
        
        Some(Self {
            pool: res,
            nic_idxs: nic_idxs,
        })
    }
}
