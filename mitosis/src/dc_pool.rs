use alloc::vec::Vec;

use os_network::rdma::dc::*;
use os_network::rdma::RawGID;
use os_network::KRdmaKit::qp::DCTargetMeta;

#[allow(unused_imports)]
use crate::linux_kernel_module;

/// The clients(children)-side DCQP pool
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

    pub fn get_ctx_id(&self, idx: usize) -> core::option::Option<usize> {
        self.nic_idxs.get(idx).map(|v| *v)
    }

    pub fn get_rdma_descriptor(
        &self,
        pool_id: usize,
    ) -> core::option::Option<crate::descriptors::RDMADescriptor> {
        let nic_idx = self.nic_idxs.get(pool_id)?;

        let context = unsafe { crate::get_rdma_context_ref(*nic_idx) }.unwrap();

        /*
        Some(crate::descriptors::RDMADescriptor {
            gid: RawGID::new(context.get_gid_as_string()).unwrap(),
            service_id: crate::rdma_context::SERVICE_ID_BASE + *nic_idx as u64,
            rkey: unsafe { context.get_rkey() },
        }) */
        unimplemented!();
    }
}

use os_network::Factory;

impl<'a> DCPool<'a> {
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        crate::log::info!("Start initializing client-side DCQP pool.");

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

use alloc::sync::Arc;
use os_network::rdma::dc::DCTarget;

/// The servers(Parents)-side DC targets pool
pub struct DCTargetPool {
    pool: Vec<Arc<DCTarget>>,
    keys: Vec<u32>,
}

impl DCTargetPool {
    /// Note: this initialization function must be called **after** the contexts
    /// has been properly initialized.
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        crate::log::info!("Start initializing server-side DCTarget pool.");

        let mut pool = Vec::new();
        let mut keys = Vec::new();

        for i in 0..config.init_dc_targets {
            let nic_idx = i % config.num_nics_used;
            let factory =
                unsafe { crate::get_dc_factory_ref(nic_idx).expect("Failed to get DC factory") };
            pool.push(
                factory
                    .create_target((i + 73) as _)
                    .expect("Failed to create DC Target"),
            );
            keys.push(unsafe { factory.get_context().get_rkey() });
        }
        Some(Self {
            pool: pool,
            keys: keys,
        })
    }

    pub fn pop_one(&mut self) -> core::option::Option<(Arc<DCTarget>, u32)> { 
        let target = self.pool.pop().expect("No target left");
        let key = self.keys.pop().unwrap();
        Some((target, key))
    }

    // fill the dc target pool in the background
    pub fn fill(&mut self) {
        unimplemented!();
    }
}
