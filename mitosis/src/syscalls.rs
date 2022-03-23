use crate::linux_kernel_module::bindings::*;
use crate::linux_kernel_module::c_types::*;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

#[allow(unused_imports)]
use crate::linux_kernel_module::println;

use mitosis_protocol::CALL_NIL;

pub struct SysCallsHandler {}

pub struct SysCallsService(linux_kernel_module::chrdev::Registration);

impl SysCallsService {
    pub fn new() -> linux_kernel_module::KernelResult<Self> {
        use linux_kernel_module::cstr;
        let reg = linux_kernel_module::chrdev::builder(cstr!("mitosis"), 0..1)?
            .register_device::<SysCallsHandler>(cstr!("mitosis-syscalls"))
            .build()?;
        Ok(Self(reg))
    }
}

#[allow(non_upper_case_globals)]
impl crate::linux_kernel_module::file_operations::FileOperations for SysCallsHandler {
    fn open(
        file: *mut crate::linux_kernel_module::bindings::file,
    ) -> crate::linux_kernel_module::KernelResult<Self> {
        Ok(Self {})
    }

    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        crate::log::debug!("in ioctrl");
        match cmd {
            CALL_NIL => self.handle_nil(arg), 
            _ => {
                crate::log::error!("unknown system call command ID {}", cmd);
                -1
            }
        }
    }

    fn mmap(&mut self, _vma_p: *mut vm_area_struct) -> c_int {
        unimplemented!();
    }
}

// real system call implementations
impl SysCallsHandler {
    #[inline(always)]
    fn handle_nil(&self, arg : c_ulong) -> c_long {
        crate::log::debug!("handle nil call, with arg {}", arg);
        0
    }
}
