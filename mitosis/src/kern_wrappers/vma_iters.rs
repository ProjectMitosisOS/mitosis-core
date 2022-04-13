use alloc::vec::Vec;

use super::mm::{PhyAddrType, VirtAddrType};
use super::vma::*;
use crate::bindings::*;

use crate::descriptors::FlatPageTable;

#[allow(unused_imports)]
use crate::linux_kernel_module;

pub struct VMADumpIter<'a> {
    flat_page_table: &'a mut FlatPageTable,
    count: usize,
    engine: VMWalkEngine,
}

impl<'a> VMADumpIter<'a> {
    pub fn new(pt: &'a mut FlatPageTable) -> Self {
        let mut walk: mm_walk = Default::default();
        walk.test_walk = None;
        walk.hugetlb_entry = None;

        walk.pte_entry = Some(Self::handle_pte_entry);

        Self {
            flat_page_table: pt,
            count: 0,
            engine: VMWalkEngine::new(walk),
        }
    }

    pub fn execute(&mut self, vma: &VMA) -> usize {
        // it's safe to pass self-reference here, the private is abandoned after the `self.engine.walk` call
        self.engine.walk_callbacks.private =
            self as *const _ as *mut crate::linux_kernel_module::c_types::c_void;

        unsafe { self.engine.walk(vma.get_raw_ptr()) };
        self.count
    }

    #[allow(non_upper_case_globals)]
    #[allow(unused_variables)]
    pub unsafe extern "C" fn handle_pte_entry(
        pte: *mut pte_t,
        addr: crate::linux_kernel_module::c_types::c_ulong,
        _next: crate::linux_kernel_module::c_types::c_ulong,
        walk: *mut mm_walk,
    ) -> crate::linux_kernel_module::c_types::c_int {
        let engine: &mut Self = &mut (*((*walk).private as *mut Self));

        if engine.flat_page_table.get(addr).is_some() {
            crate::log::warn!("Duplicated page table entry for addr {:x}", addr);
        }

        let phy_addr = pmem_get_phy_from_pte(pte);
        // Filter out empty PTEs
        if core::intrinsics::likely(phy_addr > 0) {
            engine
                .flat_page_table
                .add_one(addr, phy_addr);
        }
        engine.count += 1;
        0
    }
}

pub struct VMATraverseIter {
    mappings: Vec<(VirtAddrType, PhyAddrType)>,
    engine: VMWalkEngine,
}

impl VMATraverseIter {
    pub fn new() -> Self {
        let mut walk: mm_walk = Default::default();
        walk.test_walk = None;
        walk.hugetlb_entry = None;

        walk.pmd_entry = Some(Self::handle_pmd_entry);
        walk.pte_entry = Some(Self::handle_pte_entry);
        walk.pte_hole = Some(Self::handle_pte_hole);

        Self {
            mappings: Vec::new(),
            engine: VMWalkEngine::new(walk),
        }
    }

    /// do  the read here
    /// XD: as it operates on *mut vm_area_struct, should it be marked as unsafe?
    pub fn execute(&mut self, vma: &VMA) -> &Vec<(VirtAddrType, PhyAddrType)> {
        self.mappings.clear();

        // it's safe to pass self-reference here, the private is abandoned after the `self.engine.walk` call
        self.engine.walk_callbacks.private =
            self as *const _ as *mut crate::linux_kernel_module::c_types::c_void;

        unsafe { self.engine.walk(vma.get_raw_ptr()) };
        &self.mappings
    }

    #[allow(non_upper_case_globals)]
    #[allow(unused_variables)]
    pub unsafe extern "C" fn handle_pte_entry(
        pte: *mut pte_t,
        addr: crate::linux_kernel_module::c_types::c_ulong,
        _next: crate::linux_kernel_module::c_types::c_ulong,
        walk: *mut mm_walk,
    ) -> crate::linux_kernel_module::c_types::c_int {
        let engine: &mut Self = &mut (*((*walk).private as *mut Self));
        engine.mappings.push((addr, pmem_get_phy_from_pte(pte)));
        0
    }

    #[allow(non_upper_case_globals)]
    #[allow(unused_variables)]
    pub unsafe extern "C" fn handle_pte_hole(
        _addr: crate::linux_kernel_module::c_types::c_ulong,
        _next: crate::linux_kernel_module::c_types::c_ulong,
        _walk: *mut mm_walk,
    ) -> crate::linux_kernel_module::c_types::c_int {
        // No need to implement now
        0
    }

    #[allow(non_upper_case_globals)]
    #[allow(unused_variables)]
    pub unsafe extern "C" fn handle_pmd_entry(
        _pmd: *mut pmd_t,
        _addr: crate::linux_kernel_module::c_types::c_ulong,
        _next: crate::linux_kernel_module::c_types::c_ulong,
        _walk: *mut mm_walk,
    ) -> crate::linux_kernel_module::c_types::c_int {
        // No need to implement now
        0
    }
}

/// A simple wrapper for helping walking through the page tables
#[derive(Debug)]
pub(crate) struct VMWalkEngine {
    walk_callbacks: mm_walk,
}

impl VMWalkEngine {
    pub fn new(callbacks: mm_walk) -> Self {
        Self {
            walk_callbacks: callbacks,
        }
    }

    #[allow(non_camel_case_types)]
    pub unsafe fn walk(&mut self, vma: *mut vm_area_struct) {
        let vm = *vma;
        self.walk_callbacks.vma = vma;
        self.walk_callbacks.mm = vm.vm_mm;
        self.walk_inner(vma, (*vma).vm_start, (*vma).vm_end);
    }

    #[allow(non_camel_case_types)]
    unsafe fn walk_inner(
        &mut self,
        _vma: *mut vm_area_struct,
        start: crate::linux_kernel_module::c_types::c_ulong,
        end: crate::linux_kernel_module::c_types::c_ulong,
    ) {
        // FIXME: not handling the error code
        pmem_call_walk_range(start, end, &mut self.walk_callbacks as *mut _);
    }
}
