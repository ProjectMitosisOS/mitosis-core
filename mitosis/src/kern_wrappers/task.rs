use crate::bindings::{pmem_get_current_task, task_struct};

/// Simpler wrapper of the kernel's `mm_struct`
/// It provides some handy utilities written in rust
pub struct Task {
    task_inner: &'static mut task_struct,
}

#[allow(dead_code)]
impl Task {
    pub fn new() -> Self {
        unsafe {
            Self {
                task_inner: &mut *pmem_get_current_task(),
            }
        }
    }

    /// The bellow comments are taken from arch/alpha/include/uapi/asm/ptrace.h
    /// * This struct defines the way the registers are stored on the
    /// * kernel stack during a system call or other kernel entry
    pub fn get_stack_registers() -> crate::bindings::StackRegisters {
        unsafe { *crate::bindings::pmem_get_current_pt_regs() }
    }

    /// Set registers stored on the stack 
    pub fn set_stack_registers(regs: &crate::bindings::StackRegisters) {
        unsafe { core::ptr::write_volatile(crate::bindings::pmem_get_current_pt_regs(), *regs) };
    }

    pub fn get_fs() -> crate::linux_kernel_module::c_types::c_ulong {
        unsafe { crate::bindings::pmem_arch_get_my_fs() }
    }

    pub fn get_gs() -> crate::linux_kernel_module::c_types::c_ulong {
        unsafe { crate::bindings::pmem_arch_get_my_gs() }
    }

    pub fn set_fs(
        fsbase: crate::linux_kernel_module::c_types::c_ulong,
    ) -> crate::linux_kernel_module::c_types::c_long {
        unsafe { crate::bindings::pmem_arch_set_my_fs(fsbase) }
    }

    pub fn set_gs(
        gsbase: crate::linux_kernel_module::c_types::c_ulong,
    ) -> crate::linux_kernel_module::c_types::c_long {
        unsafe { crate::bindings::pmem_arch_set_my_gs(gsbase) }
    }
}

impl core::fmt::Debug for Task {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{:?}", self.task_inner)
    }
}
