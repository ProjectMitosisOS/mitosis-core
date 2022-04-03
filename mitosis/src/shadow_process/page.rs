use core::option::Option;

use os_network::bytes::BytesMut;

use crate::bindings::*;

const _4K: usize = 4096;

/// A wrapper over the original linux's page data structure
/// It will copy the page to a newly allocated one to prevent overwriting
/// Currently, we only support 4K pages
pub struct Copy4KPage {
    inner: &'static mut page, // linux data structure wrapper always use the 'static lifetime
}

impl Copy4KPage {
    pub unsafe fn new(
        user_vaddr: *mut crate::linux_kernel_module::c_types::c_void,
    ) -> Option<Self> {
        use crate::linux_kernel_module::bindings::GFP_KERNEL;
        use crate::linux_kernel_module::user_ptr::UserSlicePtr;

        let new_page_p = pmem_alloc_page(GFP_KERNEL);

        if new_page_p.is_null() {
            return None;
        }

        let new_virt = pmem_phys_to_virt(pmem_page_to_phy(new_page_p));

        UserSlicePtr::new(user_vaddr, _4K)
            .expect("should correctly read from user")
            .reader()
            .read(core::slice::from_raw_parts_mut(new_virt as *mut u8, _4K))
            .expect("cannot copy from user");

        Some(Self {
            inner: &mut *(new_page_p as *mut page),
        })
    }

    pub fn get_kva(&self) -> *mut crate::linux_kernel_module::c_types::c_void {
        unsafe { pmem_phys_to_virt(pmem_page_to_phy(self.inner as *const _ as _)) }
    }

    pub fn to_bytes(&self) -> BytesMut {
        unsafe { BytesMut::from_raw(self.get_kva() as _, _4K) }
    }
}

impl super::page_table::GetPhyAddr for Copy4KPage {
    fn get_physical_addr(&self) -> crate::kern_wrappers::mm::PhyAddrType {
        unsafe { crate::bindings::pmem_page_to_phy(self.inner as *const _ as *mut _) }
    }
}

impl Drop for Copy4KPage {
    fn drop(&mut self) {
        unsafe { pmem_free_page(self.inner as *mut _) };
    }
}
