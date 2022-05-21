pub use x86_64::{
    structures::paging::{Page, Size4KiB},
    PhysAddr, VirtAddr,
};

use alloc::boxed::Box;

use super::page_structures::*;
use crate::linux_kernel_module;

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
            l4_page_table: Box::new(PageTable::new(PageTableLevel::Four)),
        }
    }

    /// check whether all entries in the pagetable is zero
    pub fn is_empty(&self) -> bool {
        for entry in self.l4_page_table.iter() {
            if *entry != 0 {
                return false;
            }
        }
        return true;
    }

    pub fn lookup(&self, addr: VirtAddr) -> core::option::Option<PhysAddr> {
        let entry = RemotePage::containing_address(addr);
        let l3_pt = self.l4_page_table[usize::from(entry.p4_index())] as *mut PageTable;
        if l3_pt.is_null() {
            return None;
        }
        Some(PhysAddr::new(0))
    }

    /// Add a (addr, phy) mapping to the page table.
    /// Return Some(value) if there is an existing mapping.
    /// Return None means the map is successful.
    pub fn map(&mut self, addr: VirtAddr, phy: PhysAddr) -> core::option::Option<PhysAddr> {
        let entry = RemotePage::containing_address(addr);

        let l3_pt = unsafe {
            create_table(
                usize::from(entry.p4_index()),
                (&mut (*self.l4_page_table)) as _,
            )
        };
        let l2_pt = unsafe { create_table(usize::from(entry.p3_index()), l3_pt) };
        let l1_pt = unsafe { create_table(usize::from(entry.p2_index()), l2_pt) };

        let l1_pt: &mut PageTable = unsafe { &mut (*l1_pt) };

        let res = l1_pt[usize::from(entry.p1_index())];
        if res == 0 {
            l1_pt[usize::from(entry.p1_index())] = phy.as_u64();
            return None;
        }

        return Some(PhysAddr::new(res));
    }
}

/// Helper function to create or lookup the next-level page table
unsafe fn create_table(index: usize, src: *mut PageTable) -> *mut PageTable {
    let pt: &mut PageTable = &mut (*src);
    let mut next_level = pt[index] as *mut PageTable;

    if next_level.is_null() {
        next_level = Box::into_raw(Box::new(PageTable::new(
            pt.get_level().next_lower_level().unwrap(),
        )));
        crate::log::debug!("created next level {:?}", next_level);
        pt[index] = next_level as _;
    }
    next_level
}
