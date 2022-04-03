pub mod vma;
pub use vma::*;

pub mod page_table;
pub use page_table::*;

pub mod page;
pub use page::*;

use alloc::vec::Vec;

use crate::descriptors::Descriptor;
#[allow(unused_imports)]
use crate::linux_kernel_module;

#[allow(dead_code)]
pub struct ShadowProcess {
    descriptor: crate::descriptors::Descriptor,

    shadow_vmas: Vec<ShadowVMA<'static>>,
    copy_shadow_pagetable: ShadowPageTable<Copy4KPage>,
}

impl ShadowProcess {
    pub fn get_descriptor_ref(&self) -> &crate::descriptors::Descriptor {
        &self.descriptor
    }
}

impl ShadowProcess {
    pub fn new_cow(_rdma_descriptor: crate::descriptors::RDMADescriptor) -> Self {
        unimplemented!();
    }

    pub fn new_copy(rdma_descriptor: crate::descriptors::RDMADescriptor) -> Self {
        // data structures of myself
        let mut shadow_pt = ShadowPageTable::<Copy4KPage>::new();
        let mut shadow_vmas: Vec<ShadowVMA<'static>> = Vec::new();

        // data structures for descriptors
        let mut vma_descriptors = Vec::new();
        let mut pt: crate::descriptors::FlatPageTable = Default::default();

        // the generation process
        let task = crate::kern_wrappers::task::Task::new();
        let mm = task.get_memory_descriptor();

        for vma in mm.get_vma_iter() {
            vma_descriptors.push(vma.generate_descriptor());

            let s_vma = ShadowVMA::new(vma, false);
            VMACopyPTGenerater::new(&s_vma, &mut shadow_pt, &mut pt).generate();

            shadow_vmas.push(s_vma);
        }

        /*
        crate::log::debug!(
            "sanity check new shadow process, pt len {} spt len {}",
            pt.len(),
            shadow_pt.len()
        ); */

        Self {
            shadow_vmas: shadow_vmas,
            copy_shadow_pagetable: shadow_pt,
            descriptor: Descriptor {
                machine_info: rdma_descriptor,
                regs: task.generate_reg_descriptor(),
                page_table: pt,
                vma: vma_descriptors,
            },
        }
    }
}
