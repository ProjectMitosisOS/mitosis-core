#[derive(Copy, Clone, Default)]
pub struct VMADescriptor {
    pub range: (
        crate::kern_wrappers::mm::VirtAddrType,
        crate::kern_wrappers::mm::VirtAddrType,
    ), // [start,end] of a vma range (virtual address)
    pub flags: crate::bindings::vm_flags_t,
    pub prot: crate::bindings::pgprot_t,
}

impl os_network::serialize::Serialize for VMADescriptor {}