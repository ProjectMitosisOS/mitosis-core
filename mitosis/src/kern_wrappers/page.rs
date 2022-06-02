use crate::bindings::page;
use crate::kern_wrappers::mm::{PhyAddrType, VirtAddrType};
use hashbrown::HashMap;

// kernel pages reserved for CoW
pub type KPageTable = HashMap<VirtAddrType, Page>;

/// A simple wrapper over the kernel's `struct page`
pub struct Page {
    page_p: *mut page,
    raw: bool,
}

impl Page {
    pub unsafe fn new_from_raw(inner: *mut page) -> Self {
        Self {
            page_p: inner,
            raw: true,
        }
    }

    pub unsafe fn new_from_upage(
        user_page_va: *mut crate::linux_kernel_module::c_types::c_void,
    ) -> Option<Self> {
        use crate::bindings::{pmem_alloc_page, pmem_page_to_phy, pmem_phys_to_virt};
        use crate::linux_kernel_module::bindings::GFP_KERNEL;
        let new_page_p = pmem_alloc_page(GFP_KERNEL);
        if new_page_p == core::ptr::null_mut() {
            return None;
        }
        let new_virt = pmem_phys_to_virt(pmem_page_to_phy(new_page_p));
        let res =
            crate::linux_kernel_module::bindings::_copy_from_user(new_virt, user_page_va, 4096);

        if res != 0 {
            // free the page
            Self {
                page_p: new_page_p as *mut _,
                raw: false,
            };
            return None;
        }
        Some(Self {
            page_p: new_page_p as *mut _,
            raw: false,
        })
    }

    #[inline]
    pub fn get_phy(&self) -> PhyAddrType {
        unsafe { crate::bindings::pmem_page_to_phy(self.page_p as *mut _) }
    }

    #[inline]
    pub unsafe fn increase_ref_count(&mut self) {
        crate::bindings::pmem_get_page(self.page_p);
        crate::bindings::pmem_page_dup_rmap(self.page_p, false);
    }

    #[inline]
    pub fn get_inner(&self) -> *mut page {
        self.page_p
    }

    /// get a kernel accessible virtual address that of the page
    #[inline]
    pub unsafe fn get_kernel_virt(&self) -> *mut crate::linux_kernel_module::c_types::c_void {
        crate::bindings::pmem_phys_to_virt(self.get_phy())
    }
}

impl Clone for Page {
    fn clone(&self) -> Self {
        Self {
            page_p: self.page_p,
            raw: self.raw,
        }
    }
}

impl core::fmt::Debug for Page {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> core::fmt::Result {
        fmt.debug_struct("A physical page")
            .field("page", &self.page_p)
            .finish()
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        // the page is allocated from new page
        if !self.raw {
            unsafe { crate::bindings::pmem_free_page(self.page_p as *mut _) };
        }
    }
}

unsafe impl Sync for Page {}

/// copy the content of src to the dst
pub unsafe fn copy_page_content_4k(dst: *mut page, src: *mut page) {
    use crate::linux_kernel_module::c_types;
    use rust_kernel_linux_util::bindings::memcpy;

    let dst_va = crate::bindings::pmem_page_to_virt(dst) as u64;
    let src_va = crate::bindings::pmem_page_to_virt(src) as u64;

    memcpy(
        (dst_va as *mut i8).cast::<c_types::c_void>(),
        (src_va as *mut i8).cast::<c_types::c_void>(),
        4096, // 4KB
    );
}
