use hashbrown::HashMap;

use crate::descriptors::Descriptor;
use crate::shadow_process::*;

#[allow(unused_imports)]
use crate::linux_kernel_module;

use os_network::bytes::ToBytes;
use os_network::{msg::UDMsg as RMemory, serialize::Serialize};

struct ProcessBundler {
    process: ShadowProcess,
    serialized_buf: RMemory,
}

impl ProcessBundler {
    fn new(process: ShadowProcess) -> Self {
        let mut buf = RMemory::new(process.get_descriptor_ref().serialization_buf_len(), 0);
        process.get_descriptor_ref().serialize(buf.get_bytes_mut());
        Self {
            process: process,
            serialized_buf: buf,
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

        let pool_idx = unsafe { crate::bindings::pmem_get_current_cpu() };
        let rdma_descriptor =
            unsafe { crate::get_dc_pool_service_ref().get_rdma_context(pool_idx as _) }?;

        self.registered_processes.insert(
            key,
            ProcessBundler::new(crate::shadow_process::ShadowProcess::new_copy(
                rdma_descriptor,
            )),
        );

        return Some(());
    }

    pub fn unregister(&mut self, key: usize) {
        self.registered_processes.remove(&key);
    }
}
