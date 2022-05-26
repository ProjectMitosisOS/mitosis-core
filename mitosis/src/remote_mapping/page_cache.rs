use crate::bindings::*;
use crate::descriptors::PageMapAllocator;
use crate::kern_wrappers::mm::{PhyAddrType, VirtAddrType};
use crate::shadow_process::COW4KPage;
use alloc::string::{String, ToString};
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

type PageCacheKey = PhyAddrType;
type PageCacheValue = COW4KPage;

#[derive(Default)]
pub struct PageCache {
    store: HashMap<PageCacheKey, COW4KPage, DefaultHashBuilder, PageMapAllocator>,
}

pub unsafe fn copy_kernel_page(dst: *mut page, src_va: VirtAddrType) {
    use crate::linux_kernel_module::c_types;
    use rust_kernel_linux_util::bindings::memcpy;

    let dst_va = crate::bindings::pmem_page_to_virt(dst) as u64;
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
        phy_addr: PhyAddrType,
    ) -> PageCacheKey {
        // let splitter = "@";
        // remote_mac_id.to_string()
        //     + splitter
        //     + handler_id.to_string().as_str()
        //     + splitter
        //     + phy_addr.to_string().as_str()
        phy_addr
    }

    pub fn lookup_mut(
        &mut self,
        remote_mac_id: usize,
        handler_id: usize,
        phy_addr: PhyAddrType,
    ) -> Option<&mut PageCacheValue> {
        self.store
            .get_mut(&Self::gen_cache_key(remote_mac_id, handler_id, phy_addr))
    }

    pub fn lookup(
        &self,
        remote_mac_id: usize,
        handler_id: usize,
        phy_addr: PhyAddrType,
    ) -> Option<&PageCacheValue> {
        self.store
            .get(&Self::gen_cache_key(remote_mac_id, handler_id, phy_addr))
    }

    pub fn insert(
        &mut self,
        remote_mac_id: usize,
        handler_id: usize,
        phy_addr: PhyAddrType,
        page: PageCacheValue,
    ) {
        let key = Self::gen_cache_key(remote_mac_id, handler_id, phy_addr);
        self.store.insert(key, page);
    }

    pub fn entry_len(&self) -> usize {
        self.store.len()
    }
}
