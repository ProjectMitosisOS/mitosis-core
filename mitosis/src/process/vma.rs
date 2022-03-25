use crate::bindings::vm_area_struct;

#[derive(Debug, Clone, Copy)]
pub struct VMAMeta<'a> {
    vma_inner: &'a vm_area_struct,
}

#[allow(dead_code)]
impl<'a> VMAMeta<'a> {
    pub fn new(vma: &'a crate::bindings::vm_area_struct) -> Self {
        Self {
            vma_inner: vma,
        }
    }

    pub fn get_range(&self) -> (VirtAddrType, VirtAddrType) {
        (self.vma_inner.vm_start, self.vma_inner.vm_end)
    }

    pub fn is_stack(&self) -> bool {
        self.get_flags().contains(VMFlags::STACK)
    }

    pub fn get_start(&self) -> VirtAddrType {
        self.vma_inner.vm_start
    }

    pub fn get_end(&self) -> VirtAddrType {
        self.vma_inner.vm_end
    }

    pub fn get_sz(&self) -> u64 {
        self.vma_inner.vm_end - self.vma_inner.vm_start
    }

    pub fn get_prot(&self) -> crate::bindings::pgprot_t {
        self.vma_inner.vm_page_prot
    }

    pub fn get_flags(&self) -> crate::bindings::VMFlags {
        unsafe { crate::bindings::VMFlags::from_bits_unchecked(self.vma_inner.vm_flags) }
    }

    pub fn get_raw_flags(&self) -> crate::linux_kernel_module::c_types::c_ulong {
        self.vma_inner.vm_flags
    }

    pub fn get_mmap_flags(&self) -> crate::linux_kernel_module::c_types::c_ulong {
        let mut ret = 0;
        if self.get_flags().contains(VMFlags::READ) {
            ret |= crate::bindings::PMEM_PROT_READ;
        }
        if self.get_flags().contains(VMFlags::WRITE) {
            ret |= crate::bindings::PMEM_PROT_WRITE;
        }
        if self.get_flags().contains(VMFlags::EXEC) {
            ret |= crate::bindings::PMEM_PROT_EXEC;
        }
        if self.is_stack() {
            ret |= crate::bindings::PMEM_PROT_GROWSUP;
        }
        ret
    }

    pub fn get_all_mappings(&self) -> PageTableT {
        self.vma_inner.get_all_mappings()
    }
}

use crate::bindings::VMFlags;

impl core::fmt::Display for VMAMeta<'_> {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> core::fmt::Result {
        let vm_flags = self.get_flags();
        fmt.write_fmt(format_args!(
            "Meta vm:  exe: {} r: {} w: {}",
            vm_flags.contains(VMFlags::EXEC),
            vm_flags.contains(VMFlags::READ),
            vm_flags.contains(VMFlags::WRITE)
        ))
    }
}

use alloc::vec::Vec;

use crate::bindings::{mm_walk, pmd_t, pmem_call_walk_range, pmem_get_phy_from_pte, pte_t};

pub type VirtAddrType = u64;
pub type PhyAddrType = u64;
type PageTableT = Vec<(VirtAddrType, PhyAddrType)>;

impl vm_area_struct {
    fn get_all_mappings(&self) -> PageTableT {
        let mut vm_read = VMReadEngine::new();
        vm_read.execute(self as *const _);
        vm_read.mappings
    }
}

/// An abstraction that helps iterating the page table mapping of a vma
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
    pub fn walk(&mut self, vma: *mut vm_area_struct) {
        let vm = unsafe { *vma };

        self.walk_callbacks.vma = vma;
        self.walk_callbacks.mm = vm.vm_mm;
        unsafe { self.walk_inner(vma, (*vma).vm_start, (*vma).vm_end) };
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

pub(crate) struct VMReadEngine {
    mappings: PageTableT,
    engine: VMWalkEngine,
}

impl VMReadEngine {
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
    pub fn execute(&mut self, vma: *const vm_area_struct) {
        self.mappings.clear();

        // it's safe to pass self-reference here, the private is abandoned after the `self.engine.walk` call
        self.engine.walk_callbacks.private =
            self as *const _ as *mut crate::linux_kernel_module::c_types::c_void;

        self.engine.walk(vma as *mut _);
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

// clear all the present bit of the mappings in one vm_area_structure
pub(crate) struct VMClearEngine {
    engine: VMWalkEngine,
}

use crate::bindings::pmem_clear_pte_present;

#[allow(dead_code)]
impl VMClearEngine {
    pub fn new() -> Self {
        let mut walk: mm_walk = Default::default();
        walk.test_walk = None;
        walk.hugetlb_entry = None;

        walk.pte_entry = Some(Self::handle_pte_entry);

        Self {
            engine: VMWalkEngine::new(walk),
        }
    }

    pub fn execute(&mut self, vma: *mut vm_area_struct) {
        self.engine.walk(vma);
    }

    #[allow(non_upper_case_globals)]
    #[allow(unused_variables)]
    pub unsafe extern "C" fn handle_pte_entry(
        pte: *mut pte_t,
        addr: crate::linux_kernel_module::c_types::c_ulong,
        _next: crate::linux_kernel_module::c_types::c_ulong,
        walk: *mut mm_walk,
    ) -> crate::linux_kernel_module::c_types::c_int {
        pmem_clear_pte_present(pte);
        0
    }
}
