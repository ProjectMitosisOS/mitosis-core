use crate::bindings::*;
use crate::descriptors::PageMapAllocator;
use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::HashMap;

// Map from handler id into the whole page table
type PageCacheKey = u64;
type PageCacheValue = crate::remote_mapping::RemotePageTable;

#[derive(Default)]
pub struct PageCache {
    store: HashMap<PageCacheKey, PageCacheValue, DefaultHashBuilder, PageMapAllocator>,
}

impl PageCache {
    #[inline(always)]
    fn gen_cache_key(
        remote_mac_id: usize,
        handler_id: usize,
    ) -> PageCacheKey {
        handler_id as PageCacheKey
    }

    pub fn lookup_mut(
        &mut self,
        remote_mac_id: usize,
        handler_id: usize,
    ) -> Option<&mut PageCacheValue> {
        self.store
            .get_mut(&Self::gen_cache_key(remote_mac_id, handler_id))
    }

    pub fn lookup(
        &self,
        remote_mac_id: usize,
        handler_id: usize,
    ) -> Option<&PageCacheValue> {
        self.store
            .get(&Self::gen_cache_key(remote_mac_id, handler_id))
    }

    pub fn insert(
        &mut self,
        remote_mac_id: usize,
        handler_id: usize,
        value: PageCacheValue,
    ) {
        let key = Self::gen_cache_key(remote_mac_id, handler_id);
        self.store.insert(key, value);
    }

    pub fn entry_len(&self) -> usize {
        self.store.len()
    }
}
