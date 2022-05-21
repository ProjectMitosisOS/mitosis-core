pub use x86_64::{
    structures::paging::{Size4KiB, Page}, 
    PhysAddr, VirtAddr,
};


use alloc::boxed::Box;

use super::page_structures::*;

pub type RemotePage = Page<Size4KiB>; 

/// Abstracts a (remote) forked page table
/// We do this by emulating the four-level page table of x86
///
/// Limitation:
/// - we only support 4KiB page table now
///
#[allow(dead_code)]
#[derive(Debug)]
pub struct RemotePageTable {
    // note: we use the box as the inner data type/
    // otherwise, this data structure can easily overflow the kernel stack
    l4_page_table: Box<PageTable>,
}

impl RemotePageTable {
    /// create an empty page table
    pub fn new() -> Self {
        Self {
            l4_page_table: Box::new(PageTable::new()),
        }
    }

    /// check whether all entries in the pagetable is zero 
    pub fn is_empty(&self) -> bool { 
        for entry in self.l4_page_table.iter() { 
            if *entry != 0  { 
                return false;
            }
        }
        return true;
    }

    pub fn lookup(&self, addr : VirtAddr) -> core::option::Option<PhysAddr> { 
        let entry = RemotePage::containing_address(addr);
        let l3_pt = self.l4_page_table[usize::from(entry.p4_index())] as *mut PageTable;
        if l3_pt.is_null() { 
            return None;
        }
        Some(PhysAddr::new(0))
    }
}