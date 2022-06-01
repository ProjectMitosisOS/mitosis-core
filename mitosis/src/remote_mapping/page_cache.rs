use crate::bindings::*;
use crate::descriptors::PageMapAllocator;
use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::HashMap;

const _4K: usize = 4096;

pub struct Page {
    pub inner: *mut page,
}

impl Clone for Page {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

impl Page {
    #[inline(always)]
    pub fn get_kva(&self) -> *mut crate::linux_kernel_module::c_types::c_void {
        unsafe { pmem_phys_to_virt(pmem_page_to_phy(self.inner as *const _ as _)) }
    }
}

// Map from handler id into the whole page table
type PageCacheKey = u64;
type PageCacheValue = crate::remote_mapping::RemotePageTable;


#[derive(Default)]
pub struct PageCache {
    store: HashMap<PageCacheKey, PageCacheValue, DefaultHashBuilder, PageMapAllocator>,
}

pub unsafe fn copy_kernel_page(dst: *mut page, src: *mut page) {
    use crate::linux_kernel_module::c_types;
    use rust_kernel_linux_util::bindings::memcpy;

    let dst_va = crate::bindings::pmem_page_to_virt(dst) as u64;
    let src_va = crate::bindings::pmem_page_to_virt(src) as u64;
    memcpy(
        (dst_va as *mut i8).cast::<c_types::c_void>(),
        (src_va as *mut i8).cast::<c_types::c_void>(),
        _4K as _,
    );
}

/// Mark the target page as COW mode
pub unsafe fn mark_cow(page: *mut page) {
    crate::bindings::pmem_get_page(page);
    crate::bindings::pmem_page_dup_rmap(page, false);
}

impl PageCache {
    #[inline(always)]
    fn gen_cache_key(
        remote_mac_id: usize,
        handler_id: usize,
    ) -> PageCacheKey {
        // let splitter = "@";
        // remote_mac_id.to_string()
        //     + splitter
        //     + handler_id.to_string().as_str()
        //     + splitter
        //     + phy_addr.to_string().as_str()
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
