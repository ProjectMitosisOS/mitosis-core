use crate::*;

use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;

pub(crate) struct MySyscallHandler;

// FIXME: we need to place these with auto-generated code, e.g., proc_macros
// But currently, we don't have time to do so
#[allow(non_upper_case_globals)]
impl FileOperations for MySyscallHandler {
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
            0 => self.handle_basic(arg),
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

use alloc::vec::Vec;
use mitosis::kern_wrappers::*;
use mitosis::shadow_process::*;

// real system call implementations
impl MySyscallHandler {
    #[inline(always)]
    fn handle_basic(&self, arg: c_ulong) -> c_long {
        crate::log::debug!("handle basic tests, with arg {}", arg);

        let task = task::Task::new();
        let mm = task.get_memory_descriptor();

        let mut vma_cnt = 0;
        let mut vma_file_cnt = 0;

        for vma in mm.get_vma_iter() {
            vma_cnt += 1;
            // we must not create the shadow process for COW
            // otherwise, the kernel module will be blocked
            let s_vma = ShadowVMA::new(vma, false);
            if s_vma.backed_by_file() {
                vma_file_cnt += 1;
            }
        }
        log::debug!("vma cnt {}, backed by file cnt {}", vma_cnt, vma_file_cnt);

        0
    }
}
