use hashbrown::HashMap;

/// A data structure for RPC to lookup the descriptor 
/// It should be initialized in startup.rs
pub struct DescriptorFactoryService { 
    // TODO: shall we wrap it into a lock? since there may be multiple RPC threads
    // TODO: shall we record the serialized buffer to avoid on-the-fly serialization? 
    registered_descriptors : HashMap<usize, super::Descriptor>, 
}