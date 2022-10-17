use alloc::vec::Vec;

use rust_kernel_rdma_base::*;

use os_network::rdma::{dc::*, DCCreationMeta};

use hashbrown::HashMap;

#[allow(unused_imports)]
use crate::linux_kernel_module;

/// The clients(children)-side DCQP pool
pub struct DCPool {
    pool: Vec<DCConn>,
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
    pub fn get_dc_qp(&mut self, idx: usize) -> core::option::Option<&mut DCConn> {
        self.pool.get_mut(idx)
    }

    pub fn get_ctx_id(&self, idx: usize) -> core::option::Option<usize> {
        self.nic_idxs.get(idx).map(|v| *v)
    }

    /// Pop the DCQP and the lkey corresponding to it
    /// This function is not **thread-safe**,
    /// must be used by a single thread / protected by a mutex
    #[inline]
    pub fn pop_one_qp(&mut self) -> core::option::Option<DCConn> {
        self.pool.pop()
    }

    #[inline]
    pub fn create_one_qp(&mut self) {
        let nic_idx = 0;
        self.pool.push(
            unsafe { crate::get_dc_factory_ref(nic_idx) }
                .expect("fatal, should not fail to create dc factory")
                .create(DCCreationMeta { port: 1 }) // WTX: port is default to 1
                .expect("Failed to create DC QP")
        );
    }

    /// Arg: DCQP, its corresponding lkey
    #[inline]
    pub fn push_one_qp(&mut self, qp: DCConn) {
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
                unsafe { crate::get_dc_factory_ref(nic_idx) }
                    .expect("fatal, should not fail to create dc factory")
                    .create(
                        DCCreationMeta { port: config.default_nic_port }
                    )
                    .expect("Failed to create DC QP")
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
    pool: Vec<DCTargetMeta>,
}

/// The DCTarget and its corresponding metadata
/// We cache the lid/gid instead of querying them in any critical path
pub struct DCTargetMeta {
    pub(crate) target: Arc<DCTarget>,
    pub(crate) nic_idx: usize,
    pub(crate) rkey: u32,
    pub(crate) lid: u16,
    pub(crate) gid: ib_gid,
}

impl DCTargetPool {
    /// Note: this initialization function must be called **after** the contexts
    /// has been properly initialized.
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        crate::log::info!("Start initializing server-side DCTarget pool.");

        let mut pool = Vec::new();
        for i in 0..config.init_dc_targets {
            let nic_idx = i % config.num_nics_used;
            let factory =
                unsafe { crate::get_dc_factory_ref(nic_idx).expect("Failed to get DC factory") };
            let rkey = factory.get_context().rkey();
            let target = factory
                .create_target((i + 73) as _, config.default_nic_port)
                .expect("Failed to create DC Target");
            let meta = target
                .get_datagram_meta()
                .expect("Failed to get datagram meta from DCTarget QP");
            
            pool.push(
                DCTargetMeta {
                    target,
                    nic_idx,
                    rkey,
                    lid: meta.lid,
                    gid: meta.gid,
                }
            );
        }
        Some(Self {
            pool,
        })
    }

    pub fn pop_one(&mut self) -> core::option::Option<DCTargetMeta> {
        self.pool.pop()
    }

    // fill the dc target pool in the background
    pub fn fill(&mut self) {
        unimplemented!();
    }
}
