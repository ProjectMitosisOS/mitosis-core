use alloc::vec::Vec;

use super::vma::*;
use super::mm::{PhyAddrType,VirtAddrType};
use crate::bindings::*;

pub struct VMADumpIter {
    // not implemented, should dump to an os_network::bytes::BytesMut;
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
struct VMWalkEngine {
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