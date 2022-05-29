use core::pin::Pin;

use alloc::collections::VecDeque;

#[allow(unused_imports)]
use crate::{
    linux_kernel_module,
    remote_mapping::{PageTable, PhysAddr},
    remote_paging::{AccessInfo, DCReqPayload},
};
use os_network::Conn;
use os_network::{
    future::{Async, Poll},
    rdma::{
        dc::{DCConn, DCFactory},
        ConnErr,
    },
    Factory, Future,
};

use crate::remote_mapping::RemotePageTableIter;

use crate::bindings::page;
use crate::KRdmaKit::rust_kernel_rdma_base::bindings::*;

use super::{Prefetch, PrefetchRequests, StepPrefetcher};

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
    access_info: AccessInfo,
}

type PrefetchReq = <RemotePageTableIter as Iterator>::Item;

impl<'a> DCAsyncPrefetcher<'a> {
    pub fn new(fact: &'a DCFactory, remote_info: AccessInfo) -> Result<Self, ConnErr> {
        let lkey = unsafe { fact.get_context().get_lkey() };
        let conn = fact.create(())?;
        Ok(Self {
            conn: conn,
            lkey: lkey,
            pending_queues: Default::default(),
            access_info: remote_info,
        })
    }

    /// Number of pending prefetch requests in the queue
    pub fn num_pending(&self) -> usize {
        self.pending_queues.len()
    }

    /// Submit requests to the prefetcher executor, and execute
    /// PhysAddr format:
    /// - 0x00000000001 => during prefetch
    /// - 0xdeadbeafde1 => prefetched
    /// - 0xdeadbeafde0 => remote PA
    ///
    /// TODO:
    /// Currently, we assume that the remote PA will not change.
    /// If this is not the case, we need to 2 bits to identify
    /// whether the remote page is in the prefetch state.
    pub fn execute_reqs<P>(&mut self, mut iter: RemotePageTableIter, strategy: P)
    where
        P: Prefetch<Item = PrefetchReq>,
    {
        let reqs = strategy.generate_request(&mut iter);
        for i in 0..reqs.len() {
            // process this entry
            let pte_p = reqs[i].page;
            let pte_page = unsafe { &mut (*pte_p) };
            let phyaddr = PhysAddr::new(pte_page[reqs[i].index]);

            if phyaddr.bottom_bit() == 1 {
                // this page has been prefetched, or at least in the list
                continue;
            }

            // 1. set the page table entry's bottom bit to 1 to prevent future prefetch
            let remote_pa = pte_page[reqs[i].index];

            // FIXME: this code assumes the remote PA never changes for this children
            pte_page[reqs[i].index] = 1;

            // 2. submit the RDMA request to read the page
            let user_page =
                unsafe { crate::bindings::pmem_alloc_page(crate::bindings::PMEM_GFP_HIGHUSER) };
            let new_page_pa = unsafe { crate::bindings::pmem_page_to_phy(user_page) as u64 };

            // TODO: doorbell optimization
            let mut payload = DCReqPayload::default()
                .set_laddr(new_page_pa)
                .set_raddr(remote_pa) // copy from src into dst
                .set_sz(4096)
                .set_lkey(self.lkey)
                .set_rkey(self.access_info.rkey)
                .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
                .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
                .set_ah_ptr(unsafe { self.access_info.ah.get_inner() })
                .set_dc_access_key(self.access_info.dct_key as _)
                .set_dc_num(self.access_info.dct_num);

            let mut payload = unsafe { Pin::new_unchecked(&mut payload) };
            os_network::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());

            // crate::log::debug!("post reqs {}", self.access_info.dct_num);
            // send the requests
            self.conn.post(&payload.as_ref()).unwrap(); // FIXME: it should never fail

            // 3. record the prefetch information here
            self.pending_queues.push_back(ReplyEntry {
                pt: reqs[i].page,
                idx: reqs[i].index,
                user_page: user_page,
            });
        }
    }
}

impl<'a> Future for DCAsyncPrefetcher<'a> {
    type Output = *mut page;
    type Error = <DCConn<'a> as Future>::Error;

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        match self.conn.poll() {
            Ok(Async::Ready(_wc)) => {
                // must have one
                let v = self.pending_queues.pop_front().unwrap();
                let pte_p = v.pt;
                let pte_page = unsafe { &mut (*pte_p) };

                assert!(PhysAddr::new(v.user_page as _).bottom_bit() != 1);
                pte_page[v.idx] = (v.user_page as u64) | 1;

                return Ok(Async::Ready(v.user_page));
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => return Err(e),
        }
    }
}
