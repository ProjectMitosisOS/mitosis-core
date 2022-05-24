pub use x86_64::{
    structures::paging::{Page, Size4KiB},
    VirtAddr,
};

use alloc::boxed::Box;

use super::page_structures::*;

#[allow(unused_imports)]
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

    // number of mapped PTE in the page table
    cnt: usize,
}

impl RemotePageTable {
    /// Create an empty page table
    pub fn new() -> Self {
        Self {
            l4_page_table: Box::new(Default::default()),
            cnt: 0,
        }
    }

    /// Return the number of mapped PTEs i nthe page table
    pub fn len(&self) -> usize {
        self.cnt
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

    /// Lookup the physical address using the $addr$
    pub fn translate(&self, addr: VirtAddr) -> core::option::Option<PhysAddr> {
        let entry = RemotePage::containing_address(addr);
        let l3_pt =
            unsafe { lookup_table(usize::from(entry.p4_index()), (&(*self.l4_page_table)) as _) }?;

        let l2_pt = unsafe { lookup_table(usize::from(entry.p3_index()), l3_pt) }?;
        let l1_pt = unsafe { lookup_table(usize::from(entry.p2_index()), l2_pt) }?;
        return unsafe { lookup_table(usize::from(entry.p1_index()), l1_pt) }
            .map(|a| PhysAddr::new(a as _));
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
            self.cnt += 1;
            return None;
        }

        return Some(PhysAddr::new(res));
    }
}

/// PageTable iterator
pub struct RemotePageTableIter {
    // invariants: the cur page must be a valid level-4 page
    cur_page: *mut PageTable,
    cur_idx: usize,
}

impl RemotePageTableIter {
    pub fn new(pt: &mut RemotePageTable) -> core::option::Option<Self> {
        let mut res = Self {
            cur_page: &mut (*pt.l4_page_table) as _,
            cur_idx: 0,
        };

        res.cur_page = unsafe { Self::find_the_first_level_one_page(res.cur_page)? };
        match unsafe { res.seek_to_next_valid() } { 
            True => Some(res), 
            False => None,
        }
    }

    /// Find the next valid entry in the PTE
    /// Return true if the current is valid
    unsafe fn seek_to_next_valid(&mut self) -> bool {
        let mut cur: &mut PageTable = &mut (*self.cur_page);
        while cur[self.cur_idx] == 0 {
            self.cur_idx += 1;
            if self.cur_idx >= ENTRY_COUNT {
                // find the next page
            }
        }
        unimplemented!();
    }

    /// Find the first level one page
    unsafe fn find_the_first_level_one_page(src : *mut PageTable) -> core::option::Option<*mut PageTable> { 
        let mut cur = &mut (*src);
        while cur.get_level() != PageTableLevel::One { 
            let idx = cur.find_valid_entry(0)?;
            cur = &mut *(cur[idx] as *mut PageTable);
        }
        Some(cur)
    }

    /// Find the next level page
    /// For example, suppose our pages are:
    ///    A
    /// B<-  -> C
    /// 
    /// find_the_next_level_page(B) will return C
    /// 
    unsafe fn find_the_next_level_page(src : *mut PageTable) -> core::option::Option<*mut PageTable> { 

        // recursive done
        let src = &mut (*src);
        if src.get_upper_level_page().is_null() { 
            return None;
        }

        // this is level-one page, so the upper level page cannot be null
        let upper = &mut (*src.get_upper_level_page());

        let idx = upper.find_valid_entry(src.get_upper_level_page_index() + 1);
        if idx.is_some() { 
            // we are done
        } else {
            
        }
        unimplemented!();
    }

}

/// Helper function to create or lookup the next-level page table
#[inline]
unsafe fn create_table(index: usize, src: *mut PageTable) -> *mut PageTable {
    let pt: &mut PageTable = &mut (*src);
    let mut next_level = pt[index] as *mut PageTable;

    if next_level.is_null() {
        next_level = PageTable::new_from_upper(src, index);
        pt[index] = next_level as _;
    }
    next_level
}

/// Helper function to lookup the next-level page table
#[inline]
unsafe fn lookup_table(
    index: usize,
    src: *const PageTable,
) -> core::option::Option<*mut PageTable> {
    let pt: &PageTable = &(*src);
    let res = pt[index] as *mut PageTable;
    if res.is_null() {
        return None;
    }
    Some(res)
}

impl Default for RemotePageTable {
    fn default() -> Self {
        Self::new()
    }
}
