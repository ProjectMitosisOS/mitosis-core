use crate::bindings::VMFlags;
use crate::kern_wrappers::mm::VirtAddrType;

#[derive(Copy, Clone, Default)]
pub struct VMADescriptor {
    pub range: (
        crate::kern_wrappers::mm::VirtAddrType,
        crate::kern_wrappers::mm::VirtAddrType,
    ),
    // [start,end] of a vma range (virtual address)
    pub flags: crate::bindings::vm_flags_t,
    pub prot: crate::bindings::pgprot_t,
}

impl VMADescriptor {
    #[inline]
    pub fn is_stack(&self) -> bool {
        self.get_flags().contains(VMFlags::STACK)
    }
    #[inline]
    pub fn get_start(&self) -> VirtAddrType {
        self.range.0
    }
    #[inline]
    pub fn get_end(&self) -> VirtAddrType {
        self.range.1
    }
    #[inline]
    pub fn get_sz(&self) -> u64 {
        self.range.1 - self.range.0
    }
    #[inline]
    pub fn get_prot(&self) -> crate::bindings::pgprot_t {
        self.prot
    }
    #[inline]
    pub fn get_flags(&self) -> crate::bindings::VMFlags {
        unsafe { crate::bindings::VMFlags::from_bits_unchecked(self.flags) }
    }
    #[inline]
    pub fn get_mmap_flags(&self) -> crate::linux_kernel_module::c_types::c_ulong {
        let mut ret = 0;
        if self.get_flags().contains(VMFlags::READ) {
            ret |= crate::bindings::PMEM_PROT_READ;     // 0x01
        }
        if self.get_flags().contains(VMFlags::WRITE) {
            ret |= crate::bindings::PMEM_PROT_WRITE;    // 0x02
        }
        if self.get_flags().contains(VMFlags::EXEC) {
            ret |= crate::bindings::PMEM_PROT_EXEC;     // 0x04
        }
        if self.is_stack() {
            ret |= crate::bindings::PMEM_PROT_GROWSUP;  // 0x02000000
        }
        ret
    }
}

impl os_network::serialize::Serialize for VMADescriptor {}