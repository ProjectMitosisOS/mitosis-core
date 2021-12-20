#[allow(
    clippy::all,
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    improper_ctypes,
    non_upper_case_globals,
    dead_code
)]
mod bindings {
    use crate::linux_kernel_module::c_types;
    include!(concat!(env!("OUT_DIR"), "/bindings-swap.rs"));

    unsafe impl Send for page {}
    unsafe impl Sync for page {}
}

// #[derive(Clone, Copy)]
pub(crate) type PTRegs = pt_regs;

unsafe impl Send for vm_operations_struct {}
unsafe impl Sync for vm_operations_struct {}

pub use bindings::*;

impl core::fmt::Debug for task_struct {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> core::fmt::Result {
        fmt.debug_struct("task_struct")
            .field("mm", unsafe { &*self.mm })
            .finish()
    }
}

impl core::fmt::Display for PTRegs {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_fmt(format_args!(
            "PTRegs:  \n \
            ip 0x{:x}\n  \
            sp 0x{:x}",
            self.ip, self.sp
        ))
    }
}

impl core::fmt::Debug for mm_struct {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> core::fmt::Result {
        fmt.debug_struct("mm_struct")
            .field("mmap", &self.mmap)
            .field("total_vm", &self.total_vm)
            .finish()
    }
}

impl core::fmt::Display for vm_area_struct {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> core::fmt::Result {
        // do not use {:?} in order to avoid kernel stack overflow
        fmt.write_fmt(format_args!(
            "vm_area: 0x{:x} ~ 0x{:x}, flags: 0x{:x}, protecton: 0x{:x}",
            self.vm_start, self.vm_end, self.vm_flags, self.vm_page_prot.pgprot
        ))
    }
}

#[allow(dead_code)]
impl vm_area_struct {
    // TODO
}

bitflags::bitflags! {
    pub struct FileFlags: crate::linux_kernel_module::c_types::c_uint {
        const NONBLOCK = O_NONBLOCK;
    }
}

bitflags::bitflags! {
    pub struct PageFlags: crate::linux_kernel_module::c_types::c_ulong {
        const PRESENT = PMEM_PAGE_PRESENT;
        const RW = PMEM_PAGE_RW;
        const USER = PMEM_PAGE_USER;

        // XD: currently, I hard-coded the NX flag, since the bindgen (my version) cannot correctly generate the constants
        const NX = 1 << 63;
    }
}

bitflags::bitflags! {
    pub struct VMFlags: crate::linux_kernel_module::c_types::c_ulong {
        const READ = PMEM_VM_READ;
        const WRITE = PMEM_VM_WRITE;
        const EXEC  = PMEM_VM_EXEC;
        const STACK = PMEM_VM_STACK;
        const DONTEXPAND = PMEM_VM_DONTEXPAND;
    }
}

bitflags::bitflags! {
    pub struct FaultFlags : crate::linux_kernel_module::c_types::c_uint {
        const SIGSEGV = PMEM_VM_FAULT_SIGSEGV;
    }
}

pub struct File {
    ptr: *const bindings::file,
}

#[allow(dead_code)]
impl File {
    pub unsafe fn from_ptr(ptr: *const bindings::file) -> File {
        File { ptr }
    }

    pub fn pos(&self) -> u64 {
        unsafe { (*self.ptr).f_pos as u64 }
    }

    pub fn flags(&self) -> FileFlags {
        FileFlags::from_bits_truncate(unsafe { (*self.ptr).f_flags })
    }
}
