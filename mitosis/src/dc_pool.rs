use alloc::vec::Vec;

use os_network::rdma::dc::*;

pub struct DCPool<'a> {
    pool: Vec<DCConn<'a>>,

    // TODO: should initialize a DC Target pool
    // a simple DC key cannot protect all the stuffs in a fine-grained way
}

impl<'a> DCPool<'a> {
    pub fn get_dc_qp(&mut self, idx: usize) -> core::option::Option<&mut DCConn<'a>> {
        self.pool.get_mut(idx)
    }
}

use os_network::Factory;

impl<'a> DCPool<'a> {
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        let mut res = Vec::new();

        for i in 0..config.max_core_cnt {
            let nic_idx = i % config.num_nics_used;
            res.push(
                unsafe { crate::get_dc_factory_ref(nic_idx) }
                    .expect("fatal, should not fail to create dc factory")
                    .create(())
                    .expect("Failed to create DC QP"),
            );
        }
        Some(Self { pool: res })
    }
}
