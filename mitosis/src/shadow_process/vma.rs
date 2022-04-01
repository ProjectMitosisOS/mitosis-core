use crate::bindings::*;
use crate::kern_wrappers::vma::VMA;

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
