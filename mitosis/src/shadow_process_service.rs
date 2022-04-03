use alloc::sync::Arc;
use alloc::vec::Vec;

use hashbrown::HashMap;
use os_network::rdma::dc::DCTarget;

use crate::descriptors::RDMADescriptor;
use crate::shadow_process::*;

#[allow(unused_imports)]
use crate::linux_kernel_module;

use os_network::bytes::ToBytes;
use os_network::{msg::UDMsg as RMemory, serialize::Serialize};

struct ProcessBundler {
    process: ShadowProcess,
    serialized_buf: RMemory,
    bound_dc_targets: Vec<Arc<DCTarget>>,
}

impl ProcessBundler {
    fn new(process: ShadowProcess, targets: Arc<DCTarget>) -> Self {
        let mut buf = RMemory::new(process.get_descriptor_ref().serialization_buf_len(), 0);
        process.get_descriptor_ref().serialize(buf.get_bytes_mut());

        let mut bound_targets = Vec::new();
        bound_targets.push(targets);

        Self {
            process: process,
            serialized_buf: buf,
            bound_dc_targets: bound_targets,
        }
    }
}

pub struct ShadowProcessService {
    registered_processes: HashMap<usize, ProcessBundler>,
}

impl ShadowProcessService {
    pub fn new() -> Self {
        Self {
            registered_processes: Default::default(),
        }
    }

    pub fn query_descriptor(
        &self,
        key: usize,
    ) -> core::option::Option<&crate::descriptors::Descriptor> {
        self.registered_processes
            .get(&key)
            .map(|s| s.process.get_descriptor_ref())
    }

    pub fn add_myself_copy(&mut self, key: usize) -> core::option::Option<()> {
        if self.registered_processes.contains_key(&key) {
            crate::log::warn!(
                "Failed to prepare: the register key {} has already been taken. ",
                key
            );
            return None;
        }

        let (target, descriptor) = RDMADescriptor::new_from_dc_target_pool()?;

        self.registered_processes.insert(
            key,
            ProcessBundler::new(
                crate::shadow_process::ShadowProcess::new_copy(descriptor),
                target,
            ),
        );

        return Some(());
    }

    pub fn unregister(&mut self, key: usize) {
        self.registered_processes.remove(&key);
    }
}
