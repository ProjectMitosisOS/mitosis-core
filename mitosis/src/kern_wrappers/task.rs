use super::mm::MemoryDescriptor;
pub(crate) use crate::bindings::{pmem_get_current_task, task_struct};

#[allow(unused_imports)]
use crate::linux_kernel_module;

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

use crate::descriptors::*;

impl Task {
    /// Unmap all of the VMA in the task
    #[inline]
    pub fn unmap_self(&self) {
        let mut md = self.get_memory_descriptor();
        self.get_memory_descriptor().get_vma_iter().for_each(|m| {
            md.unmap_region(m.get_start() as _, m.get_sz() as _);
        });
    }

    /// Map one region into current task
    #[inline]
    pub unsafe fn map_one_region(
        &self,
        file: *mut crate::bindings::file,
        vma_meta: &VMADescriptor,
        next_vma: core::option::Option<&VMADescriptor>,
    ) -> Option<&'static mut crate::bindings::vm_area_struct> {
        use crate::bindings::{pmem_vm_mmap, VMFlags};

        let ret = {
            // we need to extend the VMA of heap & stack to avoid corrupting
            let mut extended_map_area_sz = 1024 * 1024 * 1024; // 1GB
            // let mut extended_map_area_sz = 0;
            if next_vma.is_some() {
                assert!(next_vma.unwrap().get_start() >= vma_meta.get_end());
                extended_map_area_sz = core::cmp::min(
                    extended_map_area_sz,
                    next_vma.unwrap().get_start() - vma_meta.get_end(),
                );
            } else { 
                extended_map_area_sz = 0;
            }

            if vma_meta.is_anonymous() {
                pmem_vm_mmap(
                    file,
                    vma_meta.get_start(),
                    vma_meta.get_sz() + extended_map_area_sz,
                    vma_meta.get_mmap_flags(),
                    crate::kern_wrappers::mm::mmap_flags::MAP_PRIVATE,
                    0,
                )
            } else {
                pmem_vm_mmap(
                    file,
                    vma_meta.get_start(),
                    vma_meta.get_sz(),
                    vma_meta.get_mmap_flags(),
                    crate::kern_wrappers::mm::mmap_flags::MAP_PRIVATE,
                    0,
                )
            }
        };
        if ret != vma_meta.get_start() {
            return None;
        }
        let vma = self
            .get_memory_descriptor()
            .find_vma(vma_meta.get_start())
            .unwrap();
        if vma_meta.is_stack() {
            vma.vm_flags = (VMFlags::from_bits_unchecked(vma.vm_flags) | VMFlags::STACK).bits();
        } else {
            vma.vm_flags =
                (VMFlags::from_bits_unchecked(vma.vm_flags) | VMFlags::DONTEXPAND).bits();
        }
        #[cfg(feature = "eager-resume")]
        {
            vma.vm_flags = (crate::bindings::VMFlags::from_bits_unchecked(vma.vm_flags)
                | crate::bindings::VMFlags::MIXEDMAP)
                .bits();
        }
        return Some(vma);
    }

    #[inline]
    pub fn set_mm_reg_states(&mut self, regs: &RegDescriptor) {
        self.get_memory_descriptor().flush_tlb_all();
        self.set_stack_registers(&regs.others);
        self.set_tls_fs(regs.fs);
        self.set_tls_gs(regs.gs);
    }

    pub fn generate_reg_descriptor(&self) -> RegDescriptor {
        RegDescriptor {
            others: self.get_stack_registers(),
            fs: self.get_tls_fs(),
            gs: self.get_tls_gs(),
        }
    }

    pub fn generate_mm(&self) -> (alloc::vec::Vec<VMADescriptor>, FlatPageTable) {
        use crate::kern_wrappers::vma_iters::VMADumpIter;

        let mut pt: FlatPageTable = Default::default();
        let mut vmas = alloc::vec::Vec::new();

        let mm = self.get_memory_descriptor();
        let vma_iters = mm.get_vma_iter();

        let mut total_counts = 0;
        for vma in vma_iters {
            vmas.push(vma.generate_descriptor());
            total_counts += VMADumpIter::new(&mut pt).execute(&vma);
        }
        crate::log::debug!("Total {} pages touched", total_counts);

        (vmas, pt)
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

    /// The below comments are from <https://stackoverflow.com/questions/6611346/how-are-the-fs-gs-registers-used-in-linux-amd64>
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
