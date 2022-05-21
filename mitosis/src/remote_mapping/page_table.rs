pub use x86_64::{
    structures::paging::{Size4KiB, Page}, 
    PhysAddr, VirtAddr,
};

use alloc::boxed::Box;

use super::page_structures::*;

pub type RemotePage = Page<Size4KiB>; 

/// Abstracts a (remote) forked page table
///
/// Limitation:
/// - we only support 4KiB page table now
///
#[allow(dead_code)]
#[derive(Debug)]
pub struct RemotePageTable {
    // note: we use the box as the inner data type/
    // otherwise, this data structure can easily overflow the kernel stack
    inner_page_table: Box<PageTable>,
}

impl RemotePageTable {
    /// create an empty page table
    pub fn new() -> Self {
        Self {
            inner_page_table: Box::new(PageTable::new()),
        }
    }

    /// check whether all entries in the pagetable is zero 
    pub fn is_empty(&self) -> bool { 
        for entry in self.inner_page_table.iter() { 
            if *entry != 0  { 
                return false;
            }
        }
        return true;
    }
}
