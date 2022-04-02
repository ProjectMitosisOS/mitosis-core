pub mod vma; 
pub use vma::*;

pub mod page_table;
pub use page_table::*;

pub mod page;
pub use page::*;

use alloc::vec::Vec;

pub struct ShadowProcess { 
    descriptor : crate::descriptors::Descriptor, 
    
    shadow_vmas : Vec<ShadowVMA<'static>>,
    copy_shadow_pagetable : ShadowPageTable<Copy4KPage>
}

impl ShadowProcess { 

    pub fn new_cow(_rdma_descriptor : crate::descriptors::RDMADescriptor) -> Self { 
        unimplemented!();
    }

    pub fn new_copy(rdma_descriptor : crate::descriptors::RDMADescriptor) -> Self { 
        // data structures of myself
        let mut shadow_pt = ShadowPageTable::<Copy4KPage>::new();
        let shadow_vmas : Vec<ShadowVMA::<'static>> = Vec::new();

        // data structures for descriptors 
        let mut vma_descriptors = Vec::new();

        // the generation process
        let task = crate::kern_wrappers::task::Task::new();
        let mm = task.get_memory_descriptor();

        for vma in mm.get_vma_iter() { 
            vma_descriptors.push(vma.generate_descriptor());
            
            let s_vma = ShadowVMA::new(vma, false);            
        }

        unimplemented!();
    }
}