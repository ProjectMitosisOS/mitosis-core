use crate::bindings::{vm_area_struct, VMFlags, file};

use super::mm::VirtAddrType;

#[derive(Debug)]
pub struct VMA<'a> {
    vma_inner: &'a mut vm_area_struct,
}

impl<'a> VMA<'a> {
    pub fn generate_descriptor(&self) -> crate::descriptors::VMADescriptor {
        crate::descriptors::VMADescriptor {
            range: self.get_range(),
            flags: self.get_raw_flags(),
            prot: self.get_prot(),
        }
    }
}

use alloc::string::String;
use rust_kernel_linux_util::string::ptr2string;

impl<'a> VMA<'a> {
    pub fn new(vma: &'a mut crate::bindings::vm_area_struct) -> Self {
        Self { vma_inner: vma }
    }

    pub fn get_range(&self) -> (VirtAddrType, VirtAddrType) {
        (self.get_start(), self.get_end())
    }

    /// whether this VMA is a stack
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

    pub fn set_raw_flags(&mut self, flags : crate::linux_kernel_module::c_types::c_ulong) {
        self.vma_inner.vm_flags = flags;
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

    pub unsafe fn get_raw_ptr(&self) -> *mut vm_area_struct {
        self.vma_inner as *const vm_area_struct as _
    }

    pub unsafe fn get_file_ptr(&self) -> *mut file { 
        self.vma_inner.vm_file
    }

    pub unsafe fn get_backed_file_name(&self) -> core::option::Option<String> {
        if self.vma_inner.vm_file != core::ptr::null_mut() {
            let file = *(self.vma_inner.vm_file);
            let dentry = *(file.f_path.dentry);

            return Some(ptr2string(&dentry.d_iname as *const _));
        }
        None
    }
}

impl core::fmt::Display for VMA<'_> {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> core::fmt::Result {
        // let vm_flags = self.get_flags();
        fmt.write_fmt(format_args!(
            "VMA 0x{:x}~0x{:x}, sz: {}, file: {:?}",
            self.get_start(),
            self.get_end(),
            self.get_sz(),
            unsafe { self.get_backed_file_name() },
        ))
    }
}
