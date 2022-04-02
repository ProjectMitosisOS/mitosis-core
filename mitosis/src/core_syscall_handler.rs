use core::option::Option;

use crate::linux_kernel_module::c_types::*;
use crate::syscalls::FileOperations;

#[allow(unused_imports)]
use crate::linux_kernel_module;

/// The MitosisSysCallService has the following two jobs:
///  1. handle up parent/child system calls
///  2. register the corresponding pagefault handler
///
pub struct MitosisSysCallHandler {
    caller_status: (), // structure to encapsulate caller's status
    remote_data: Option<()>,
    my_file: *mut crate::bindings::file,
}

#[allow(non_upper_case_globals)]
impl FileOperations for MitosisSysCallHandler {
    #[inline]
    fn open(
        file: *mut crate::linux_kernel_module::bindings::file,
    ) -> crate::linux_kernel_module::KernelResult<Self> {
        unsafe {
            MY_VM_OP = Default::default();
            MY_VM_OP.open = Some(open_handler);
            MY_VM_OP.fault = Some(page_fault_handler);
            MY_VM_OP.access = None;
        };

        Ok(Self {
            my_file: file as *mut _,
            caller_status: (),
            remote_data: None,
        })
    }

    #[allow(non_snake_case)]
    #[inline]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        match cmd {
            mitosis_protocol::CALL_PREPARE => self.syscall_prepare(arg),
            _ => {
                crate::log::error!("unknown system call command ID {}", cmd);
                -1
            }
        }
    }

    #[inline]
    fn mmap(
        &mut self,
        vma_p: *mut rust_kernel_linux_util::linux_kernel_module::bindings::vm_area_struct,
    ) -> c_int {
        unsafe {
            (*vma_p).vm_private_data = (self as *mut Self).cast::<c_void>();
            (*vma_p).vm_ops = &mut MY_VM_OP as *mut crate::bindings::vm_operations_struct as *mut _;
        }
        0
    }
}

/// The system call parts
impl MitosisSysCallHandler {
    fn syscall_prepare(&mut self, arg: c_ulong) -> c_long {
        crate::log::debug!("In prepare systemcall handler");
        0
    }
}

/// The fault handler parts
static mut MY_VM_OP: crate::bindings::vm_operations_struct = unsafe {
    core::mem::transmute([0u8; core::mem::size_of::<crate::bindings::vm_operations_struct>()])
};

#[allow(dead_code)]
unsafe extern "C" fn open_handler(_area: *mut crate::bindings::vm_area_struct) {}

#[allow(dead_code)]
unsafe extern "C" fn page_fault_handler(vmf: *mut crate::bindings::vm_fault) -> c_int {
    let handler: *mut MitosisSysCallHandler = (*(*vmf).vma).vm_private_data as *mut _;
    (*handler).handle_page_fault(vmf)
}

impl MitosisSysCallHandler {
    #[inline(always)]
    unsafe fn handle_page_fault(&mut self, vmf: *mut crate::bindings::vm_fault) -> c_int {
        unimplemented!();
    }
}

unsafe impl Sync for MitosisSysCallHandler {}

unsafe impl Send for MitosisSysCallHandler {}
