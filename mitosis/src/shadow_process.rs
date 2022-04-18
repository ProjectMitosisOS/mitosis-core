pub mod vma;

pub use vma::*;

pub mod page_table;

pub use page_table::*;

pub mod page;

pub use page::*;

use crate::descriptors::VMAPageTable;
use alloc::vec::Vec;

#[allow(unused_imports)]
use crate::linux_kernel_module;

#[cfg(feature = "fast-descriptors")]
type Descriptor = crate::descriptors::FastDescriptor;
#[cfg(not(feature = "fast-descriptors"))]
type Descriptor = crate::descriptors::Descriptor;

#[allow(dead_code)]
pub struct ShadowProcess {
    descriptor: Descriptor,

    shadow_vmas: Vec<ShadowVMA<'static>>,

    // COW shadow page table is only needed.
    // However, for testing purposes, we need to maintain the copy page table.
    // FIXME: maybe we should use enum for this?
    copy_shadow_pagetable: core::option::Option<ShadowPageTable<Copy4KPage>>,
    cow_shadow_pagetable: core::option::Option<ShadowPageTable<COW4KPage>>,
}

impl ShadowProcess {
    pub fn get_descriptor_ref(&self) -> &Descriptor {
        &self.descriptor
    }
}

#[cfg(not(feature = "fast-descriptors"))]
impl ShadowProcess {
    /// Crate a new shadow processing by marking all the
    /// memories of the original one to copy-on-write(COW).
    pub fn new_cow(rdma_descriptor: crate::descriptors::RDMADescriptor) -> Self {
        let mut shadow_pt = ShadowPageTable::<COW4KPage>::new();
        let mut shadow_vmas: Vec<ShadowVMA<'static>> = Vec::new();

        let mut vma_descriptors = Vec::new();
        let mut pt: crate::descriptors::FlatPageTable = Default::default();

        // the generation process
        let task = crate::kern_wrappers::task::Task::new();
        let mut mm = task.get_memory_descriptor();

        for vma in mm.get_vma_iter() {
            vma_descriptors.push(vma.generate_descriptor());

            let mut s_vma = ShadowVMA::new(vma, true);
            VMACOWPTGenerator::new(&s_vma, &mut shadow_pt, &mut pt).generate();

            shadow_vmas.push(s_vma);
        }
        // clear the TLB

        mm.flush_tlb_mm();

        // crate::log::debug!("Check flat page table sz: {}", pt.len());
        Self {
            shadow_vmas,
            cow_shadow_pagetable: Some(shadow_pt),
            copy_shadow_pagetable: None,
            descriptor: Descriptor {
                machine_info: rdma_descriptor,
                regs: task.generate_reg_descriptor(),
                page_table: pt,
                vma: vma_descriptors,
            },
        }
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
            VMACopyPTGenerator::new(&s_vma, &mut shadow_pt, &mut pt).generate();

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
            copy_shadow_pagetable: Some(shadow_pt),
            cow_shadow_pagetable: None,
            descriptor: Descriptor {
                machine_info: rdma_descriptor,
                regs: task.generate_reg_descriptor(),
                page_table: pt,
                vma: vma_descriptors,
            },
        }
    }
}

#[cfg(feature = "fast-descriptors")]
impl ShadowProcess {
    /// Crate a new shadow processing by marking all the
    /// memories of the original one to copy-on-write(COW).
    pub fn new_cow(rdma_descriptor: crate::descriptors::RDMADescriptor) -> Self {
        let mut shadow_pt = ShadowPageTable::<COW4KPage>::new();
        let mut shadow_vmas: Vec<ShadowVMA<'static>> = Vec::new();

        let mut vma_descriptors = Vec::new();
        let mut vma_page_table: Vec<VMAPageTable> = Vec::new();
        // the generation process
        let task = crate::kern_wrappers::task::Task::new();
        let mut mm = task.get_memory_descriptor();

        for vma in mm.get_vma_iter() {
            vma_descriptors.push(vma.generate_descriptor());
            shadow_vmas.push(ShadowVMA::new(vma, true));
            vma_page_table.push(Default::default());
        }

        for (idx, _) in mm.get_vma_iter().enumerate() {
            let mut pt: &mut VMAPageTable = vma_page_table.get_mut(idx).unwrap();
            let mut s_vma = shadow_vmas.get(idx).unwrap();
            VMACOWPTGenerator::new(s_vma, &mut shadow_pt, pt).generate();
        }
        // clear the TLB
        mm.flush_tlb_mm();
        Self {
            shadow_vmas,
            cow_shadow_pagetable: Some(shadow_pt),
            copy_shadow_pagetable: None,
            descriptor: Descriptor {
                machine_info: rdma_descriptor,
                regs: task.generate_reg_descriptor(),
                // page_table: Vec::default(), // FIXME: why this latency is so high ?
                page_table: vma_page_table,
                vma: vma_descriptors,
            },
        }
    }

    pub fn new_copy(rdma_descriptor: crate::descriptors::RDMADescriptor) -> Self {
        // // data structures of myself
        // let mut shadow_pt = ShadowPageTable::<Copy4KPage>::new();
        // let mut shadow_vmas: Vec<ShadowVMA<'static>> = Vec::new();
        //
        // // data structures for descriptors
        // let mut vma_descriptors = Vec::new();
        // let mut pt: crate::descriptors::FlatPageTable = Default::default();
        //
        // // the generation process
        // let task = crate::kern_wrappers::task::Task::new();
        // let mm = task.get_memory_descriptor();
        //
        // for vma in mm.get_vma_iter() {
        //     vma_descriptors.push(vma.generate_descriptor());
        //
        //     let s_vma = ShadowVMA::new(vma, false);
        //     VMACopyPTGenerator::new(&s_vma, &mut shadow_pt, &mut pt).generate();
        //
        //     shadow_vmas.push(s_vma);
        // }
        //
        // /*
        // crate::log::debug!(
        //     "sanity check new shadow process, pt len {} spt len {}",
        //     pt.len(),
        //     shadow_pt.len()
        // ); */
        //
        // Self {
        //     shadow_vmas: shadow_vmas,
        //     copy_shadow_pagetable: Some(shadow_pt),
        //     cow_shadow_pagetable: None,
        //     descriptor: Descriptor {
        //         machine_info: rdma_descriptor,
        //         regs: task.generate_reg_descriptor(),
        //         page_table: pt,
        //         vma: vma_descriptors,
        //     },
        // }
        todo!()
    }
}
