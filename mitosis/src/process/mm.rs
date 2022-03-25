use crate::bindings::mm_struct;
use crate::process::vma::VirtAddrType;

#[allow(unused_imports)]
use crate::linux_kernel_module::println;

/// taken from /include/uapi/linux/mman.h in the linux kernel
#[allow(dead_code)]
pub mod flags {
    pub const MAP_SHARED: crate::linux_kernel_module::c_types::c_ulong = 0x01;
    pub const MAP_PRIVATE: crate::linux_kernel_module::c_types::c_ulong = 0x02;
}

/// Simpler wrapper of the kernel's `mm_struct`
/// It provides some handy utilities written in rust
#[derive(Debug)]
pub struct MemoryManage {
    mm_inner: &'static mut mm_struct,
}

use crate::bindings::{pmem_flush_tlb_all, vm_area_struct};

#[allow(dead_code)]
impl MemoryManage {
    pub unsafe fn new(mm_ptr: *mut mm_struct) -> Self {
        Self {
            mm_inner: &mut (*mm_ptr),
        }
    }

    pub fn get_vm_iters(&self) -> MemoryManageIter {
        MemoryManageIter::new(self)
    }

    pub fn find_vma(
        &self,
        addr: VirtAddrType,
    ) -> core::option::Option<&'static mut vm_area_struct> {
        let vma_p =
            unsafe { crate::bindings::find_vma(self.mm_inner as *const _ as *mut mm_struct, addr) };
        if vma_p == core::ptr::null_mut() {
            return None;
        }
        return unsafe { Some(&mut (*vma_p)) };
    }

    #[allow(dead_code)]
    pub fn unmap_region(
        &mut self,
        addr_s: VirtAddrType,
        sz: usize,
    ) -> crate::linux_kernel_module::c_types::c_int {
        use crate::bindings::pmem_do_munmap;
        unsafe { pmem_do_munmap(self.get_mm_inner(), addr_s, sz, core::ptr::null_mut()) }
    }

    // According to:  https://www.kernel.org/doc/html/latest/core-api/cachetlb.html
    #[allow(dead_code)]
    pub fn flush_tlb(&mut self) {
        unsafe { pmem_flush_tlb_all() };
    }
}

impl MemoryManage {
    /// find a specific memory range descriptor
    ///
    /// # Arguments
    ///
    /// * `idx` - index to the `mm_struct`'s mmap list
    #[allow(dead_code)]
    pub fn get_vma_area(&self, idx: usize) -> core::option::Option<*mut vm_area_struct> {
        let mut start = 0;
        let mut cur = self.mm_inner.mmap;
        while start != idx {
            start += 1;
            cur = unsafe { (*cur).vm_next };
        }
        if cur != core::ptr::null_mut() {
            Some(cur)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn get_mm_inner(&mut self) -> *mut mm_struct {
        self.mm_inner as *mut _
    }
}

pub struct MemoryManageIter {
    cur: *mut vm_area_struct,
}

// uses an iterator to simplfiy memory range traversal
impl Iterator for MemoryManageIter {
    type Item = &'static mut vm_area_struct;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == core::ptr::null_mut() {
            None
        } else {
            let res = self.cur;
            self.cur = unsafe { (*res).vm_next };
            unsafe { Some(&mut (*res)) }
        }
    }
}

impl MemoryManageIter {
    pub fn new(m: &MemoryManage) -> Self {
        Self {
            cur: m.mm_inner.mmap,
        }
    }
}
