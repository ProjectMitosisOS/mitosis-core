//! Abstractions for faked remote  page tables and other paging related structures.
//!
//! Page tables translate virtual memory “pages” to remote physical memory.
//!
//! Credits: some code is taken from https://github.com/rust-osdev/x86_64/blob/master/src/structures/paging/mod.rs

pub use page_structures::*;
pub use page_table::{RemotePage, RemotePageTable, RemotePageTableIter};

pub mod page_structures;
pub mod page_table;
