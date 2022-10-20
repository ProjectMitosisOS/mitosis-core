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
    fn ioctrl(&mut self, cmd: c_uint, _arg: c_ulong) -> c_long {
        match cmd {
            0 => self.test_task(),
            1 => self.test_mm(),
            // it seems that 2 is omitted by ioctrl
            3 => self.test_vma(),
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

use mitosis::kern_wrappers::vma_iters::*;
use mitosis::kern_wrappers::*;

// real system call implementations
impl MySyscallHandler {
    fn test_task(&self) -> c_long {
        crate::log::info!("test task");

        let task = task::Task::new();
        crate::log::info!("get task {:?}", task);

        let pt_regs = task::Task::new().get_stack_registers();
        crate::log::info!("sanity check stack registers {:?}", pt_regs);
        0
    }

    fn test_mm(&self) -> c_long {
        crate::log::info!("test mm");
        let task = task::Task::new();
        let mm = task.get_memory_descriptor();
        let vma_iters = mm.get_vma_iter();

        let mut count = 0;
        for _ in vma_iters {
            count += 1;
        }
        crate::log::info!("mm get {} vmas", count);
        0
    }

    fn test_vma(&self) -> c_long {
        crate::log::info!("test vma");

        let task = task::Task::new();
        let mm = task.get_memory_descriptor();
        let vma_iters = mm.get_vma_iter();

        let mut stack_page_count = 0;
        let mut total_page_count = 0;

        for vma in vma_iters {
            if vma.is_stack() {
                stack_page_count += 1;
            }

            total_page_count += VMATraverseIter::new().execute(&vma).len();
        }
        crate::log::info!("stack page count: {}", stack_page_count);
        crate::log::info!("total page count: {}", total_page_count);
        0
    }
}
