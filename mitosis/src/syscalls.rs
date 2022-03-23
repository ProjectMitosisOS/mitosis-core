use crate::linux_kernel_module::bindings::*;
use crate::linux_kernel_module::c_types::*;

pub struct SysCallsService {}

#[allow(non_upper_case_globals)]
impl crate::linux_kernel_module::file_operations::FileOperations for ModuleManager {
    fn open(
        file: *mut crate::linux_kernel_module::bindings::file,
    ) -> crate::linux_kernel_module::KernelResult<Self> {
        Ok(Self)
    }

    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        match cmd {}
    }
}
