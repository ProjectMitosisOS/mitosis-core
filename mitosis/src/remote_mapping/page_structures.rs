//! Abstractions for page tables and page table entries.
//! 
//! Credits: https://github.com/rust-osdev/x86_64/blob/master/src/structures/paging/page_table.rs

use core::fmt;
use core::ops::{Index, IndexMut};

pub use x86_64::{
    structures::paging::{Size4KiB, Page}, 
    PhysAddr, VirtAddr,
};

/// The number of entries in a page table.
const ENTRY_COUNT: usize = 512;

pub type PageTableEntry = u64;

/// Represents a <K (u64), V> mapping using the page table structure
///
/// Always page-sized.
///
/// This struct implements the `Index` and `IndexMut` traits, so the entries can be accessed
/// through index operations. For example, `page_table[15]` returns the 15th page table entry.
#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    /// Creates an empty page table.
    #[inline]
    pub const fn new() -> Self {
        const EMPTY: PageTableEntry = 0;
        PageTable {
            entries: [EMPTY; ENTRY_COUNT],
        }
    }

    /// Returns an iterator over the entries of the page table.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        self.entries.iter()
    }

    /// Returns an iterator that allows modifying the entries of the page table.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        self.entries.iter_mut()
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl Index<PageTableIndex> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[usize::from(index)]
    }
}

impl IndexMut<PageTableIndex> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[usize::from(index)]
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PageTable {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries[..].fmt(f)
    }
}

/// A 9-bit index into a page table.
///
/// Can be used to select one of the 512 entries of a page table.
///
/// Guaranteed to only ever contain 0..512.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    /// Creates a new index from the given `u16`. Panics if the given value is >=512.
    #[inline]
    pub fn new(index: u16) -> Self {
        assert!(usize::from(index) < ENTRY_COUNT);
        Self(index)
    }

    /// Creates a new index from the given `u16`. Throws away bits if the value is >=512.
    #[inline]
    pub const fn new_truncate(index: u16) -> Self {
        Self(index % ENTRY_COUNT as u16)
    }
}

impl From<PageTableIndex> for u16 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        index.0
    }
}

impl From<PageTableIndex> for u32 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        u32::from(index.0)
    }
}

impl From<PageTableIndex> for u64 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        u64::from(index.0)
    }
}

impl From<PageTableIndex> for usize {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        usize::from(index.0)
    }
}

/// A 12-bit offset into a 4KiB Page.
///
/// This type is returned by the `VirtAddr::page_offset` method.
///
/// Guaranteed to only ever contain 0..4096.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageOffset(u16);

impl PageOffset {
    /// Creates a new offset from the given `u16`. Panics if the passed value is >=4096.
    #[inline]
    pub fn new(offset: u16) -> Self {
        assert!(offset < (1 << 12));
        Self(offset)
    }

    /// Creates a new offset from the given `u16`. Throws away bits if the value is >=4096.
    #[inline]
    pub const fn new_truncate(offset: u16) -> Self {
        Self(offset % (1 << 12))
    }
}

impl From<PageOffset> for u16 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        offset.0
    }
}

impl From<PageOffset> for u32 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        u32::from(offset.0)
    }
}

impl From<PageOffset> for u64 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        u64::from(offset.0)
    }
}

impl From<PageOffset> for usize {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        usize::from(offset.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A value between 1 and 4.
pub enum PageTableLevel {
    /// Represents the level for a page table.
    One = 1,
    /// Represents the level for a page directory.
    Two,
    /// Represents the level for a page-directory pointer.
    Three,
    /// Represents the level for a page-map level-4.
    Four,
}

impl PageTableLevel {
    /// Returns the next lower level or `None` for level 1
    pub const fn next_lower_level(self) -> Option<Self> {
        match self {
            PageTableLevel::Four => Some(PageTableLevel::Three),
            PageTableLevel::Three => Some(PageTableLevel::Two),
            PageTableLevel::Two => Some(PageTableLevel::One),
            PageTableLevel::One => None,
        }
    }

    /// Returns the alignment for the address space described by a table of this level.
    pub const fn table_address_space_alignment(self) -> u64 {
        1u64 << (self as u8 * 9 + 12)
    }

    /// Returns the alignment for the address space described by an entry in a table of this level.
    pub const fn entry_address_space_alignment(self) -> u64 {
        1u64 << (((self as u8 - 1) * 9) + 12)
    }
}