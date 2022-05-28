use core::slice::SliceIndex;

use alloc::collections::VecDeque;

use crate::remote_mapping::PageTable;
use os_network::{
    rdma::{
        dc::{DCConn, DCFactory},
        ConnErr,
    },
    Factory,
};

use crate::remote_mapping::RemotePageTableIter;

use crate::bindings::page;

use super::{PrefetchRequests, StepPrefetcher};

/// This struct is really, really, unsafe
/// Since I currently don't know how to do it right in rust
/// I will come back to this issue later
pub struct ReplyEntry {
    pt: *mut PageTable, // page table to update
    idx: usize,
    user_page: *mut page, // user page to hold the requests
}

/// Each DCAsyncPrefetcher has a DCConn responsible for executing the async RDMA requests
pub struct DCAsyncPrefetcher<'a> {
    conn: DCConn<'a>,
    lkey: u32,
    pending_queues: VecDeque<ReplyEntry>,
}

type PrefetchReq = <RemotePageTableIter as Iterator>::Item;

impl<'a> DCAsyncPrefetcher<'a> {
    pub fn new(fact: &'a DCFactory) -> Result<Self, ConnErr> {
        let lkey = unsafe { fact.get_context().get_lkey() };
        let conn = fact.create(())?;
        Ok(Self {
            conn: conn,
            lkey: lkey,
            pending_queues: Default::default(),
        })
    }

    /// Submit requests to the prefetcher executor, and execute
    pub fn execute_reqs(
        &mut self,
        mut iter: RemotePageTableIter,
        mut strategy: StepPrefetcher<PrefetchReq>,
    ) {
        let reqs = strategy.generate_request(&mut iter);
        for i in 0..reqs.len() {            
            self.pending_queues.push_back(ReplyEntry {
                pt: reqs[i].page,
                idx: reqs[i].index,
                user_page: core::ptr::null_mut(),
                // user_page: crate::bindings::pmem_alloc_page(crate::bindings::PMEM_GFP_HIGHUSER),
            });
        }
    }
}
