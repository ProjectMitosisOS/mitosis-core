use crate::bindings::*;
use crate::kern_wrappers::mm::VirtAddrType;
use crate::kern_wrappers::vma::VMA;

#[allow(unused_imports)]
use crate::linux_kernel_module;

use super::{COW4KPage, Copy4KPage, GetPhyAddr};

/// The shadow VMA is just a wrapper over the original process's VMA
/// The difference is that, upon creation, it will change the process's
/// map flag to SHARED to allow COW. It will also increase the reference counter
/// of files (if any) mapped by this VMA, similar to a local fork.
pub struct ShadowVMA<'a> {
    vma_inner: VMA<'a>,
    shadow_file: *mut file,
    is_cow: bool,
}

impl<'a> ShadowVMA<'a> {
    pub fn new(mut vma: VMA<'a>, is_cow: bool) -> Self {
        // increment the file reference counter

        let file = unsafe { vma.get_file_ptr() };
        if !file.is_null() && is_cow {
            unsafe { pmem_get_file(file) };
        }

        // toggle the VM map flag
        let mut vm_flag = vma.get_flags();
        if vm_flag.contains(VMFlags::WRITE) && is_cow {
            vm_flag.insert(VMFlags::SHARED);
            vma.set_raw_flags(vm_flag.bits());
        }

        Self {
            vma_inner: vma,
            is_cow: is_cow,
            shadow_file: file,
        }
    }

    pub fn backed_by_file(&self) -> bool {
        !self.shadow_file.is_null()
    }

    pub fn has_write_permission(&self) -> bool {
        self.vma_inner.get_flags().contains(VMFlags::WRITE)
            || self.vma_inner.get_flags().contains(VMFlags::MAY_WRITE)
    }
}

impl Drop for ShadowVMA<'_> {
    fn drop(&mut self) {
        if !self.shadow_file.is_null() && self.is_cow {
            // crate::log::debug!("In drop shadow file {:?}", self.shadow_file);
            unsafe { pmem_put_file(self.shadow_file) };
        }
    }
}

type CopyPageTable = super::page_table::ShadowPageTable<Copy4KPage>;
type COWPageTable = super::page_table::ShadowPageTable<COW4KPage>;

pub(crate) struct VMACopyPTGenerator<'a, 'b> {
    vma: &'a ShadowVMA<'a>,
    inner: &'b mut CopyPageTable,
    inner_flat: &'b mut crate::descriptors::VMAPageTable,
}

impl<'a, 'b> VMACopyPTGenerator<'a, 'b> {
    pub fn new(
        vma: &'a ShadowVMA,
        inner: &'b mut CopyPageTable,
        inner_flat: &'b mut crate::descriptors::VMAPageTable,
    ) -> Self {
        Self {
            vma: vma,
            inner: inner,
            inner_flat: inner_flat,
        }
    }
}

use crate::kern_wrappers::vma_iters::*;

impl VMACopyPTGenerator<'_, '_> {
    pub fn generate(&self) {
        let mut walk: mm_walk = Default::default();
        walk.pte_entry = Some(Self::handle_pte_entry);
        walk.private = self as *const _ as *mut crate::linux_kernel_module::c_types::c_void;

        let mut engine = VMWalkEngine::new(walk);
        unsafe { engine.walk(self.vma.vma_inner.get_raw_ptr()) };

        // crate::log::debug!("walk done");
    }

    #[allow(non_upper_case_globals)]
    #[allow(unused_variables)]
    pub unsafe extern "C" fn handle_pte_entry(
        pte: *mut pte_t,
        addr: crate::linux_kernel_module::c_types::c_ulong,
        _next: crate::linux_kernel_module::c_types::c_ulong,
        walk: *mut mm_walk,
    ) -> crate::linux_kernel_module::c_types::c_int {
        let my: &mut Self = &mut (*((*walk).private as *mut Self));

        let phy_addr = pmem_get_phy_from_pte(pte);
        if phy_addr > 0 {
            let copied_page = Copy4KPage::new(addr as _).expect("Fail to copy from user space");
            // my.inner_flat.add_one(addr, copied_page.get_physical_addr());
            {
                let start = my.vma.vma_inner.get_start();
                my.inner_flat
                    .add_one((addr as VirtAddrType - start) as _, copied_page.get_physical_addr() as _);
            }
            // the page table is present
            my.inner.add_page(copied_page);
        }
        0
    }
}

/// This iterator will traverse the pages of VMA,
/// marks all the page to COW, and store the references in a page table
pub(crate) struct VMACOWPTGenerator<'a, 'b> {
    vma: &'a ShadowVMA<'a>,
    inner: &'b mut COWPageTable,
    inner_flat: &'b mut crate::descriptors::VMAPageTable,
}

impl<'a, 'b> VMACOWPTGenerator<'a, 'b> {
    pub fn new(
        vma: &'a ShadowVMA,
        inner: &'b mut COWPageTable,
        inner_flat: &'b mut crate::descriptors::VMAPageTable,
    ) -> Self {
        Self {
            vma,
            inner,
            inner_flat,
        }
    }
}

impl VMACOWPTGenerator<'_, '_> {
    pub fn generate(&self) {
        let mut walk: mm_walk = Default::default();
        walk.pte_entry = Some(Self::handle_pte_entry);
        walk.private = self as *const _ as *mut crate::linux_kernel_module::c_types::c_void;

        let mut engine = VMWalkEngine::new(walk);
        unsafe { engine.walk(self.vma.vma_inner.get_raw_ptr()) };
    }

    #[allow(non_upper_case_globals)]
    #[allow(unused_variables)]
    pub unsafe extern "C" fn handle_pte_entry(
        pte: *mut pte_t,
        addr: crate::linux_kernel_module::c_types::c_ulong,
        _next: crate::linux_kernel_module::c_types::c_ulong,
        walk: *mut mm_walk,
    ) -> crate::linux_kernel_module::c_types::c_int {
        use core::intrinsics::{likely, unlikely};
        let my: &mut Self = &mut (*((*walk).private as *mut Self));

        let phy_addr = pmem_get_phy_from_pte(pte);
        if likely(phy_addr > 0) {
            if unlikely(my.vma.has_write_permission()) {
                my.inner
                    .add_page(COW4KPage::new(pmem_pte_to_page(pte)).unwrap());
                pmem_clear_pte_write(pte);
            }
            // #[cfg(not(feature = "fast-descriptors"))]
            // my.inner_flat.add_one(addr, phy_addr);
            // #[cfg(feature = "fast-descriptors")]
            {
                let start = my.vma.vma_inner.get_start();
                my.inner_flat
                    .add_one((addr as VirtAddrType - start) as _, phy_addr as _);
            }
        }
        0
    }
}
