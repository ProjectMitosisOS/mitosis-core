use crate::bindings::*;
use crate::descriptors::PageMapAllocator;
use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::HashMap;

// Map from handler id into the whole page table
type Key = u64;
type Value = crate::remote_mapping::RemotePageTable;

/// A global kernel-space KV store that stores 
/// a mapping between: handler_id -> its page table, 
/// whose page table entries may reference to a local cache
/// TODO: we can abstract it to a more general KV 
#[derive(Default)]
pub struct RemotePageTableCache {
    store: HashMap<Key, Value, DefaultHashBuilder, PageMapAllocator>,
}

impl RemotePageTableCache {
    #[inline(always)]
    fn gen_cache_key(
        _remote_mac_id: usize,
        handler_id: usize,
    ) -> Key {
        handler_id as Key
    }

    pub fn lookup_mut(
        &mut self,
        remote_mac_id: usize,
        handler_id: usize,
    ) -> Option<&mut Value> {
        self.store
            .get_mut(&Self::gen_cache_key(remote_mac_id, handler_id))
    }

    pub fn lookup(
        &self,
        remote_mac_id: usize,
        handler_id: usize,
    ) -> Option<&Value> {
        self.store
            .get(&Self::gen_cache_key(remote_mac_id, handler_id))
    }

    pub fn insert(
        &mut self,
        remote_mac_id: usize,
        handler_id: usize,
        value: Value,
    ) {
        let key = Self::gen_cache_key(remote_mac_id, handler_id);
        self.store.insert(key, value);
    }

    pub fn num(&self) -> usize {
        self.store.len()
    }
}
