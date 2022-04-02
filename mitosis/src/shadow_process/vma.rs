use crate::bindings::*;
use crate::kern_wrappers::vma::VMA;

use super::Copy4KPage;

/// The shadow VMA is just a wrapper over the original process's VMA
/// The difference is that, upon creation, it will change the process's
/// map flag to SHARED to allow COW. It will also increase the reference counter
/// of files (if any) mapped by this VMA, similar to a local fork.
pub struct ShadowVMA<'a> {
    vma_inner: VMA<'a>,
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
        }
    }

    pub fn backed_by_file(&self) -> bool {
        !unsafe { self.vma_inner.get_file_ptr().is_null() }
    }
}

impl Drop for ShadowVMA<'_> {
    fn drop(&mut self) {
        unsafe {
            if !self.vma_inner.get_file_ptr().is_null() && self.is_cow {
                pmem_put_file(self.vma_inner.get_file_ptr());
            }
        }
    }
}

type CopyPageTable = super::page_table::ShadowPageTable<Copy4KPage>;

pub(crate) struct VMACopyPTGenerater<'a, 'b> {
    vma: &'a ShadowVMA<'a>,
    inner: &'b mut CopyPageTable,
    inner_flat: &'b mut crate::descriptors::FlatPageTable,
}

impl<'a, 'b> VMACopyPTGenerater<'a, 'b> {
    pub fn new(
        vma: &'a ShadowVMA,
        inner: &'b mut CopyPageTable,
        inner_flat: &'b mut crate::descriptors::FlatPageTable,
    ) -> Self {
        Self {
            vma: vma,
            inner: inner,
            inner_flat: inner_flat,
        }
    }
}

use crate::kern_wrappers::vma_iters::*;

impl VMACopyPTGenerater<'_, '_> {
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
        let my: &mut Self = &mut (*((*walk).private as *mut Self));

        let phy_addr = pmem_get_phy_from_pte(pte);
        if phy_addr > 0 {
            // the page table is present
            my.inner_flat.add_one(addr, phy_addr);
            my.inner
                .add_page(Copy4KPage::new(addr as _).expect("Fail to copy from user space"));
        }
        0
    }
}
