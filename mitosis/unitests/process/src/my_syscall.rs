use crate::*;

use crate::linux_kernel_module::c_types::*;
use crate::linux_kernel_module::bindings::vm_area_struct;

pub(crate) struct MySyscallHandler;

const CALL_NIL: u32 = 0;

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
            CALL_NIL => self.test_process(arg),
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

use mitosis::process::Process;
use mitosis::process::vma::VMAMeta;

// real system call implementations
impl MySyscallHandler {
    #[inline(always)]
    fn test_process(&self, _arg: c_ulong) -> c_long {
        let process = Process::new();
        let mm = process.get_task().get_mm();

        let mut stack_page_count = 0;
        let mut total_page_count = 0;
        let mut vma_count = 0;

        // count the page and vma in the loop
        for vma in mm.get_vm_iters() {
            let vma_meta = VMAMeta::new(vma);
            if vma_meta.is_stack() {
                stack_page_count += 1;
            }

            for _ in vma_meta.get_all_mappings() {
                total_page_count += 1;
            }
            vma_count += 1;
        }
        crate::log::info!("stack page count: {}", stack_page_count);
        crate::log::info!("total page count: {}", total_page_count);
        crate::log::info!("vma count: {}", vma_count);
        0
    }
}
