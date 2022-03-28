use super::mm::MemoryDescriptor;
pub(crate) use crate::bindings::{pmem_get_current_task, task_struct};

/// Simpler wrapper of the kernel's `task_struct`
/// The task_struct structure contains all the information about a process.
/// It transfers some low-level primitives to high-level rust abstractions
pub struct Task {
    task_inner: &'static mut task_struct,
}

impl Task {
    pub fn get_memory_descriptor(&self) -> MemoryDescriptor {
        unsafe { MemoryDescriptor::new(self.task_inner.mm) }
    }
}

use crate::descriptors::reg::RegDescriptor;

impl Task {
    pub fn generate_reg_descriptor(&self) -> RegDescriptor {
        RegDescriptor {
            others: self.get_stack_registers(),
            fs: self.get_tls_fs(),
            gs: self.get_tls_gs(),
        }
    }
}

#[allow(dead_code)]
impl Task {
    pub unsafe fn new_from_raw(ptr: *mut task_struct) -> Self {
        Self {
            task_inner: &mut *ptr,
        }
    }

    pub fn new() -> Self {
        unsafe { Self::new_from_raw(pmem_get_current_task()) }
    }

    /// The bellow comments are taken from arch/alpha/include/uapi/asm/ptrace.h
    /// * This struct defines the way the registers are stored on the
    /// * kernel stack during a system call or other kernel entry
    pub fn get_stack_registers(&self) -> crate::bindings::StackRegisters {
        unsafe { *crate::bindings::pmem_get_current_pt_regs() }
    }

    /// Set registers stored on the stack
    pub fn set_stack_registers(&mut self, regs: &crate::bindings::StackRegisters) {
        unsafe { core::ptr::write_volatile(crate::bindings::pmem_get_current_pt_regs(), *regs) };
    }

    /// The below comments are from https://stackoverflow.com/questions/6611346/how-are-the-fs-gs-registers-used-in-linux-amd64
    /// "Glibc makes its TLS entry point to a struct pthread that contains some internal structures for threading.
    /// Glibc usually refers to a struct pthread variable as pd, presumably for pthread descriptor."
    /// The following is the getter/setter of the two important points:
    /// * fs register
    /// * gs register
    pub fn get_tls_fs(&self) -> crate::linux_kernel_module::c_types::c_ulong {
        unsafe { crate::bindings::pmem_arch_get_my_fs() }
    }

    pub fn get_tls_gs(&self) -> crate::linux_kernel_module::c_types::c_ulong {
        unsafe { crate::bindings::pmem_arch_get_my_gs() }
    }

    pub fn set_tls_fs(
        &mut self,
        fsbase: crate::linux_kernel_module::c_types::c_ulong,
    ) -> crate::linux_kernel_module::c_types::c_long {
        unsafe { crate::bindings::pmem_arch_set_my_fs(fsbase) }
    }

    pub fn set_tls_gs(
        &mut self,
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
