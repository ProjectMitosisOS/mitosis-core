use hashbrown::HashMap;
use crate::descriptors::Descriptor;

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
    pub fn get_descriptor(&self, key: usize) -> Option<&Descriptor> {
        self.registered_descriptors.get(&key)
    }
}