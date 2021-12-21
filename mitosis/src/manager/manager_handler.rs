use crate::linux_kernel_module::{println};
use crate::manager::manager::MY_VM_OP;
use crate::MitosisManager;
use crate::linux_kernel_module::c_types::{c_int, c_uint, c_ulong, c_long};

#[allow(dead_code)]
unsafe fn init() {
    MY_VM_OP = Default::default();
    MY_VM_OP.open = Some(open_handler);
    MY_VM_OP.fault = Some(fault_handler);
    MY_VM_OP.access = None;
}

#[allow(dead_code)]
unsafe extern "C" fn open_handler(_area: *mut crate::bindings::vm_area_struct) {
    // open handler is currently not used
}

#[allow(dead_code)]
unsafe extern "C" fn fault_handler(vmf: *mut crate::bindings::vm_fault) -> c_int {
    // TODO
    0
}

impl crate::linux_kernel_module::file_operations::FileOperations for MitosisManager {
    fn open(file: *mut crate::linux_kernel_module::bindings::file) -> crate::linux_kernel_module::KernelResult<Self> {
        unsafe { init() };
        Ok(Self{})
    }

    fn mmap(&mut self, vma_p: *mut KRdmaKit::rust_kernel_rdma_base::linux_kernel_module::bindings::vm_area_struct) -> c_int {
        // TODO
        0
    }

    #[inline(always)]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        // TODO
        println!("receive ioctrl requests");
        0
    }
}

impl Drop for MitosisManager {
    fn drop(&mut self) {
        println!("MitosisManager dropped");
    }
}
