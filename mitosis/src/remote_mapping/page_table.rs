use rust_kernel_rdma_base::VmallocAllocator;
pub use x86_64::{
    structures::paging::{Page, Size4KiB},
    VirtAddr,
};

use alloc::boxed::Box;

use super::page_structures::*;

#[allow(unused_imports)]
use crate::linux_kernel_module;

pub type RemotePageAddr = Page<Size4KiB>;

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
    l4_page_table: Box<PageTable, VmallocAllocator>,

    // number of mapped PTE in the page table
    cnt: usize,
}

impl RemotePageTable {
    #[inline]
    pub fn copy(&self) -> Self {
        Self {
            l4_page_table: self.l4_page_table.deep_copy(),
            cnt: self.cnt,
        }
    }

    /// Create an empty page table
    pub fn new() -> Self {
        Self {
            l4_page_table: Box::new_in(Default::default(), VmallocAllocator),
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
    #[inline(always)]
    pub fn translate(&self, addr: VirtAddr) -> core::option::Option<PhysAddr> {
        let (pt, index) = self.find_l1_page_idx(addr)?;
        let pt = unsafe { &mut (*pt) };
        if pt[index] != 0 {
            Some(PhysAddr::new(pt[index]))
        } else {
            None
        }
    }

    /// Lookup the last-level page of the requested address
    /// Return:
    /// - Page ptr, Entry index
    #[inline(always)]
    pub fn find_l1_page_idx(
        &self,
        addr: VirtAddr,
    ) -> core::option::Option<(*mut PageTable, usize)> {
        let entry = RemotePageAddr::containing_address(addr);
        let l3_pt =
            unsafe { lookup_table(usize::from(entry.p4_index()), (&(*self.l4_page_table)) as _) }?;

        let l2_pt = unsafe { lookup_table(usize::from(entry.p3_index()), l3_pt) }?;
        let l1_pt = unsafe { lookup_table(usize::from(entry.p2_index()), l2_pt) }?;
        return Some((l1_pt, usize::from(entry.p1_index())));
    }

    /// Add a (addr, phy) mapping to the page table.
    /// Return Some(value) if there is an existing mapping.
    /// Return None means the map is successful.
    #[inline]
    pub fn map(&mut self, addr: VirtAddr, phy: PhysAddr) -> core::option::Option<PhysAddr> {
        let entry = RemotePageAddr::containing_address(addr);

        let l1_pt = self.map_to_the_l1(&entry);
        let l1_pt: &mut PageTable = unsafe { &mut (*l1_pt) };

        let res = l1_pt[usize::from(entry.p1_index())];
        if res == 0 {
            // The bottom bit of a physical page cannot be 1 (4KB aligned)
            // We will encode the remote information in this bit
            // assert!(phy.bottom_bit() == false);
            l1_pt[usize::from(entry.p1_index())] = phy.as_u64();
            self.cnt += 1;
            return None;
        }

        return Some(PhysAddr::new(res));
    }

    /// Add one (addr, phy) mapping into the page table.
    /// The new pair mapping would **always overwrite** the origin pair (if exist)
    #[inline]
    pub fn force_map(&mut self, addr: VirtAddr, phy: PhysAddr) {
        let entry = RemotePageAddr::containing_address(addr);

        let l1_pt = self.map_to_the_l1(&entry);
        let l1_pt: &mut PageTable = unsafe { &mut (*l1_pt) };

        let res = l1_pt[usize::from(entry.p1_index())];

        // force update mapping
        l1_pt[usize::from(entry.p1_index())] = phy.as_u64();
        // not exist
        if res == 0 {
            self.cnt += 1;
        }
    }

    fn map_to_the_l1(&mut self, entry: &RemotePageAddr) -> *mut PageTable {
        let l3_pt = unsafe {
            create_table(
                usize::from(entry.p4_index()),
                (&mut (*self.l4_page_table)) as _,
            )
        };
        let l2_pt = unsafe { create_table(usize::from(entry.p3_index()), l3_pt) };
        unsafe { create_table(usize::from(entry.p2_index()), l2_pt) }
    }
}

/// PageTable iterator
#[derive(Debug)]
pub struct RemotePageTableIter {
    // invariants: the cur page must be a valid level-4 page
    cur_page: *mut PageTable,
    cur_idx: isize,
}

#[derive(Debug, Clone, Copy)]
pub struct PageEntry {
    pub page: *mut PageTable,
    pub index: usize,
    pub addr: PhysAddr,
}

impl PageEntry {
    #[inline]
    fn new(p: *mut PageTable, index: usize, addr: PhysAddr) -> Self {
        Self {
            page: p,
            index: index,
            addr: addr,
        }
    }
}

impl crate::prefetcher::NeedPrefetch for PageEntry {
    fn need_prefetch(&self) -> bool {
        self.addr.bottom_bit() == false
    }
}

impl Default for PageEntry {
    fn default() -> Self {
        Self {
            page: core::ptr::null_mut(),
            index: 0,
            addr: PhysAddr::new(0),
        }
    }
}

impl Iterator for RemotePageTableIter {
    // pointer to the page table, page table entry index, the physical page
    type Item = PageEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut cur_page = unsafe { &mut (*self.cur_page) };
        loop {
            self.cur_idx += 1;

            if self.cur_idx >= (ENTRY_COUNT as isize) {
                // we need to find another page
            } else {
                let idx = cur_page.find_valid_entry(self.cur_idx as _);
                if idx.is_some() {
                    // done
                    self.cur_idx = idx.unwrap() as isize;
                    return Some(PageEntry::new(
                        self.cur_page,
                        idx.unwrap(),
                        PhysAddr::new(cur_page[idx.unwrap()]),
                    ));
                }
            }

            // we should go to the next page
            self.cur_idx = -1;
            let next_page = unsafe { Self::find_the_next_neighbour(self.cur_page)? };
            self.cur_page = next_page;
            cur_page = unsafe { &mut (*next_page) };
        }
    }
}

impl RemotePageTableIter {
    /// Creating iterators over the L4 page table is always unsafe,
    /// because it elide the rust lifetime checks
    pub unsafe fn new(pt: &RemotePageTable) -> core::option::Option<Self> {
        let mut res = Self {
            // !! The real dangerous code here! 
            cur_page: &(*(pt.l4_page_table)) as *const PageTable as _,
            cur_idx: -1,
        };

        res.cur_page = Self::find_the_first_level_one_page(res.cur_page)?;
        Some(res)
    }

    pub unsafe fn clone(&self) -> Self {
        Self {
            cur_page: self.cur_page,
            cur_idx: self.cur_idx,
        }
    }

    /// Directly initialize from a l4 page.
    /// Note that we don't check the correctness of the passed arguments,
    /// So this function is unsafe.
    pub unsafe fn new_from_l1(l4_page: *mut PageTable, index: usize) -> Self {
        Self {
            cur_page: l4_page,
            cur_idx: index as _,
        }
    }

    /// Find the first level one page
    unsafe fn find_the_first_level_one_page(
        src: *mut PageTable,
    ) -> core::option::Option<*mut PageTable> {
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
    unsafe fn find_the_next_neighbour(src: *mut PageTable) -> core::option::Option<*mut PageTable> {
        // recursion done
        let src = &mut (*src);
        if src.get_upper_level_page().is_null() {
            return None;
        }

        // this is level-one page, so the upper level page cannot be null
        let upper = &mut (*src.get_upper_level_page());

        let idx = upper.find_valid_entry(src.get_upper_level_page_index() + 1);
        if idx.is_some() {
            // we are done, simple case
            Some(upper[idx.unwrap()] as _)
        } else {
            // we need find another neighbour of the upper
            let upper = Self::find_the_next_neighbour(src.get_upper_level_page())?;
            let upper = &mut (*upper);

            // there must be a valid one according to the nature of
            let idx = upper.find_valid_entry(0)?;
            Some(upper[idx] as _)
        }
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

impl core::fmt::Display for RemotePageTable {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if let Some(iter) = unsafe { RemotePageTableIter::new(self) } {
            for item in iter {
                if item.addr.is_cache() {
                    write!(
                        f,
                        "[table] index:{}, addr:{}, cache bit:{}",
                        item.index,
                        item.addr.as_u64(),
                        item.addr.is_cache()
                    )?;
                }
            }
        }
       Ok(()) 
    }    
}
