use crate::*;

use crate::linux_kernel_module::c_types::*;
use crate::linux_kernel_module::bindings::vm_area_struct;

pub(crate) struct MySyscallHandler;

// FIXME: we need to place these with auto-generated code, e.g., proc_macros
// But currently, we don't have time to do so
#[allow(non_upper_case_globals)]
impl FileOperations for MySyscallHandler
{
    #[inline]
    fn open(
        _file: *mut crate::linux_kernel_module::bindings::file,
    ) -> crate::linux_kernel_module::KernelResult<Self> {
        Ok(Self)
    }

    #[allow(non_snake_case)]
    #[inline]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        crate::log::debug!("in ioctrl");
        match cmd {
            _CALL_NIL => self.handle_nil(arg),
            _ => {
                crate::log::error!("unknown system call command ID {}", cmd);
                -1
            }
        }
    }

    #[inline]
    fn mmap(&mut self, _vma_p: *mut vm_area_struct) -> c_int {
        unimplemented!();
    }
}

// real system call implementations
impl MySyscallHandler {
    #[inline(always)]
    fn handle_nil(&self, arg: c_ulong) -> c_long {
        crate::log::debug!("handle nil call, with arg {}", arg);
        0
    }
}
