use hashbrown::HashMap;
use crate::descriptors::{Descriptor, RDMADescriptor};
use crate::kern_wrappers::Process;

/// A data structure for RPC to lookup the descriptor 
/// It should be initialized in startup.rs
pub struct DescriptorFactoryService {
    // TODO: shall we wrap it into a lock? since there may be multiple RPC threads
    // TODO: shall we record the serialized buffer to avoid on-the-fly serialization? 
    registered_descriptors: HashMap<usize, super::Descriptor>,
}

impl DescriptorFactoryService {
    pub fn create() -> Self {
        Self {
            // TODO: Lock maybe time consuming ? => do not lock on read
            registered_descriptors: Default::default()
        }
    }

    #[inline(always)]
    pub fn get_descriptor_ref(&self, key: usize) -> Option<&Descriptor> {
        self.registered_descriptors.get(&key)
    }

    #[inline(always)]
    pub fn get_descriptor_mut(&mut self, key: usize) -> Option<&mut Descriptor> {
        self.registered_descriptors.get_mut(&key)
    }


    #[inline(always)]
    pub fn put_current_descriptor(
        &mut self,
        key: usize,
        machine_info: RDMADescriptor) -> Option<&Descriptor> {
        let process = Process::new_from_current();
        let task = process.get_task();
        let (vma, pg_table) = task.generate_mm();
        let res = Descriptor {
            regs: task.generate_reg_descriptor(),
            page_table: pg_table,
            vma,
            machine_info,
        };
        if self.get_descriptor_ref(key).is_some() {
            // should not assign twice
            None
        } else {
            self.registered_descriptors.insert(key, res);
            self.get_descriptor_ref(key)
        }
    }
}