use alloc::vec::Vec;

use os_network::rdma::{dc::*, DCCreationMeta};

use hashbrown::HashMap;

#[allow(unused_imports)]
use crate::linux_kernel_module;

/// The clients(children)-side DCQP pool
pub struct DCPool {
    pool: Vec<Arc<DCConn>>,
    nic_idxs: Vec<usize>,
}

pub struct AccessInfoPool {
    pool: Vec<HashMap<usize, crate::remote_paging::AccessInfo>>,
}

// TODO: currently the mapping is not done good
impl AccessInfoPool {
    pub fn new(sz: usize) -> Self {
        let mut res = Vec::new();
        for _ in 0..sz {
            res.push(Default::default());
        }

        Self { pool: res }
    }

    pub fn query(
        &self,
        idx: usize,
        id: usize,
    ) -> core::option::Option<&crate::remote_paging::AccessInfo> {
        self.pool[idx].get(&id)
    }

    pub fn insert(&mut self, idx: usize, id: usize, access: crate::remote_paging::AccessInfo) {
        self.pool[idx].insert(id, access);
    }
}

impl Drop for AccessInfoPool {
    fn drop(&mut self) {
        /* 
        for pool in &mut self.pool {
            for (_, v) in pool {
                unsafe { v.ah.free() };
            }
        } */
    }
}

impl DCPool {
    pub fn get_dc_qp(&mut self, idx: usize) -> core::option::Option<&Arc<DCConn>> {
        self.pool.get(idx)
    }

    pub fn get_ctx_id(&self, idx: usize) -> core::option::Option<usize> {
        self.nic_idxs.get(idx).map(|v| *v)
    }

    /// Pop the DCQP and the lkey corresponding to it
    /// This function is not **thread-safe**,
    /// must be used by a single thread / protected by a mutex
    #[inline]
    pub fn pop_one_qp(&mut self) -> core::option::Option<Arc<DCConn>> {
        self.pool.pop()
    }

    #[inline]
    pub fn create_one_qp(&mut self) {
        let nic_idx = 0;
        self.pool.push(
            Arc::new(unsafe { crate::get_dc_factory_ref(nic_idx) }
                .expect("fatal, should not fail to create dc factory")
                .create(DCCreationMeta { port: 1 }) // WTX: port is default to 1
                .expect("Failed to create DC QP"))
        );
    }

    /// Arg: DCQP, its corresponding lkey
    #[inline]
    pub fn push_one_qp(&mut self, qp: Arc<DCConn>) {
        self.pool.push(qp)
    }
}

use os_network::Factory;

impl DCPool {
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        crate::log::info!("Start initializing client-side DCQP pool.");

        let mut res = Vec::new();
        let mut nic_idxs = Vec::new();

        for i in 0..config.max_core_cnt {
            let nic_idx = i % config.num_nics_used;
            res.push(
                Arc::new(unsafe { crate::get_dc_factory_ref(nic_idx) }
                    .expect("fatal, should not fail to create dc factory")
                    .create(
                        DCCreationMeta { port: config.default_nic_port }
                    )
                    .expect("Failed to create DC QP")),
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
/// FIXME: need lock protection
pub struct DCTargetPool {
    pool: Vec<Arc<DCTarget>>,
    nic_idxs: Vec<usize>,
    keys: Vec<u32>,
}

impl DCTargetPool {
    /// Note: this initialization function must be called **after** the contexts
    /// has been properly initialized.
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        crate::log::info!("Start initializing server-side DCTarget pool.");

        let mut pool = Vec::new();
        let mut keys = Vec::new();
        let mut nic_idxs = Vec::new();

        for i in 0..config.init_dc_targets {
            let nic_idx = i % config.num_nics_used;
            let factory =
                unsafe { crate::get_dc_factory_ref(nic_idx).expect("Failed to get DC factory") };
            pool.push(
                factory
                    .create_target((i + 73) as _, config.default_nic_port)
                    .expect("Failed to create DC Target"),
            );
            keys.push(factory.get_context().rkey());
            nic_idxs.push(nic_idx);
        }
        Some(Self {
            pool: pool,
            keys: keys,
            nic_idxs: nic_idxs,
        })
    }

    pub fn pop_one(&mut self) -> core::option::Option<(Arc<DCTarget>, usize, u32)> {
        let target = self.pool.pop().expect("No target left");
        let key = self.keys.pop().unwrap();
        let idx = self.nic_idxs.pop().unwrap();

        Some((target, idx, key))
    }

    // fill the dc target pool in the background
    pub fn fill(&mut self) {
        unimplemented!();
    }
}
