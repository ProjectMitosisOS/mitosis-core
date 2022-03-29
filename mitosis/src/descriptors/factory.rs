use alloc::sync::Arc;
use hashbrown::HashMap;
use crate::descriptors::{Descriptor, RDMADescriptor, VMADescriptor};
use crate::kern_wrappers::Process;
use crate::kern_wrappers::task::Task;
use crate::linux_kernel_module::println;

/// A data structure for RPC to lookup the descriptor 
/// It should be initialized in startup.rs
pub struct DescriptorFactoryService {
    // TODO: shall we wrap it into a lock? since there may be multiple RPC threads
    // TODO: shall we record the serialized buffer to avoid on-the-fly serialization? 
    registered_descriptors: HashMap<usize, super::Descriptor>,
}

impl DescriptorFactoryService {
    pub fn create() -> Self {
        Self {
            // TODO: Lock maybe time consuming ? => do not lock on read
            registered_descriptors: Default::default()
        }
    }

    #[inline(always)]
    pub fn get_descriptor_ref(&self, key: usize) -> Option<&Descriptor> {
        self.registered_descriptors.get(&key)
    }

    #[inline(always)]
    pub fn get_descriptor_mut(&mut self, key: usize) -> Option<&mut Descriptor> {
        self.registered_descriptors.get_mut(&key)
    }


    #[inline(always)]
    pub fn put_current_descriptor(
        &mut self,
        key: usize,
        machine_info: RDMADescriptor) -> Option<&Descriptor> {
        let process = Process::new_from_current();
        let task = process.get_task();
        let (vma, pg_table) = task.generate_mm();
        let res = Descriptor {
            regs: task.generate_reg_descriptor(),
            page_table: pg_table,
            vma,
            machine_info,
        };
        if self.get_descriptor_ref(key).is_some() {
            // should not assign twice
            None
        } else {
            self.registered_descriptors.insert(key, res);
            self.get_descriptor_ref(key)
        }
    }

    #[inline(always)]
    pub fn resume_from_descriptor(file: *mut crate::bindings::file, meta: Arc<Descriptor>) {
        #[inline]
        fn unmap_all_regions() {
            let task = Task::new();
            let mut md = task.get_memory_descriptor();

            let (mm, _) = task.generate_mm();
            mm.into_iter().for_each(|m| {
                println!("unmap start:0x{:x}, sz:{}, flag: 0x{:x}", m.get_start(), m.get_sz(), m.get_mmap_flags());
                md.unmap_region(m.get_start() as _, m.get_sz() as _);
            });
        }

        #[inline(always)]
        unsafe fn map_one_region(file: *mut crate::bindings::file, m: &VMADescriptor) {
            use crate::bindings::{pmem_vm_mmap, VMFlags};
            let ret = pmem_vm_mmap(
                file,
                m.get_start(),
                m.get_sz(),
                m.get_mmap_flags(),
                crate::kern_wrappers::mm::mmap_flags::MAP_PRIVATE,
                0,
            );
            if ret != m.get_start() {
                println!("sanity check the mmap result: {}.", ret);
                println!("a map failed to resolve the correct address: 0x{:x}.", m.get_start());
                return;
            }
            println!("pmem_vm_mmap: 0x{:x}, mmap_flags: 0x{:x}", m.get_start(), m.get_mmap_flags());
            let vma = Task::new().get_memory_descriptor().find_vma(m.get_start()).unwrap();
            if m.is_stack() {
                println!("Try to add the stack flags to the vma");
                vma.vm_flags =
                    (VMFlags::from_bits_unchecked(vma.vm_flags) | VMFlags::STACK).bits();
                println!("Add stack flags to the vma successfully");
            } else {
                vma.vm_flags =
                    (VMFlags::from_bits_unchecked(vma.vm_flags) | VMFlags::DONTEXPAND).bits();
            }
        }
        println!("pg table sz:{}", meta.page_table.len());
        // =======================================================
        // 1. Unmap origin vma regions
        return;
        unmap_all_regions();
        // 2. Map new vma regions
        (&meta.vma).into_iter().for_each(|m| {
            unsafe { map_one_region(file, m) }
        });
        // 3. Flush state
        {
            let mut task = Task::new();
            task.get_memory_descriptor().flush_tlb();
            task.set_stack_registers(&meta.regs.others);
            task.set_tls_fs(meta.regs.fs);
            task.set_tls_gs(meta.regs.gs);
        }
    }
}