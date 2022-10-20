use alloc::{collections::VecDeque, sync::Arc};

#[allow(unused_imports)]
use crate::{
    linux_kernel_module,
    remote_mapping::{PageTable, PhysAddr},
    remote_paging::{AccessInfo},
};
use os_network::{Conn, KRdmaKit::{MemoryRegion, ControlpathError}, rdma::{payload::{RDMAOp, dc::DCReqPayload}, DCCreationMeta}};
use os_network::{
    future::{Async, Poll},
    rdma::{
        dc::{DCConn, DCFactory},
    },
    Factory, Future,
};

use crate::remote_mapping::{PhysAddrBitFlag, RemotePageTableIter, K_MAGIC_IN_PREFETCH};

use crate::bindings::page;
use rust_kernel_rdma_base::bindings::*;

use super::Prefetch;

/// This struct is really, really, unsafe
/// Since I currently don't know how to do it right in rust
/// I will come back to this issue later
pub struct ReplyEntry {
    pt: *mut PageTable, // page table to update
    idx: usize,
    user_page: *mut page, // user page to hold the requests
}

/// Each DCAsyncPrefetcher has a DCConn responsible for executing the async RDMA requests
pub struct DCAsyncPrefetcher {
    conn: DCConn,
    pending_queues: VecDeque<ReplyEntry>,
    access_info: AccessInfo,
}

type PrefetchReq = <RemotePageTableIter as Iterator>::Item;

impl DCAsyncPrefetcher {
    pub fn new(fact: &DCFactory, remote_info: AccessInfo) -> Result<Self, ControlpathError> {
        // Create the DC qp with the default port number 1
        Self::new_with_meta(fact, remote_info, DCCreationMeta { port: 1 })
    }

    pub fn new_with_meta(fact: &DCFactory, remote_info: AccessInfo, meta: DCCreationMeta) -> Result<Self, ControlpathError> {
        let conn = fact.create(meta)?;
        Ok(Self {
            conn: conn,
            pending_queues: Default::default(),
            access_info: remote_info,
        })
    }

    /// Clean my prefetch requests
    /// This call is necessary to drain pending RDMA requests related to this QP.
    /// After call drain_conenctions, another container can use this QP for the prefetch. 
    pub fn drain_connections(&mut self) -> Result<DCConn, <Self as Future>::Error> {
        while !self.pending_queues.is_empty() {
            // let pt = self.pending_queues.front().unwrap().pt;
            // let idx = self.pending_queues.front().unwrap().idx;

            // we don't need to clear this page now, 
            // because upon page table drop, we will free the pages
            let page = self.poll()?;
            match page {
                Async::Ready(_p) => {
                    //unsafe {                    
                        // clear the pt entry                            
                        //let mut pt = &mut (*pt);                        
                        //pt[idx] = 0;
                        //crate::bindings::pmem_free_page(p);
                    // };
                }
                _ => {}
            };
        }
        Ok(self.conn.clone())
    }

    #[inline]
    pub fn new_from_raw(conn: DCConn, access_info: AccessInfo) -> Self {
        Self {
            conn: conn,
            pending_queues: Default::default(),
            access_info: access_info,
        }
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
    #[inline]
    pub fn execute_reqs<P, const NUM : usize>(&mut self, mut iter: RemotePageTableIter, strategy: P)
    where
        P: Prefetch<NUM, Item = PrefetchReq>,
    {
        let reqs = strategy.generate_request(&mut iter);
        for i in 0..reqs.len() {
            // process this entry
            let pte_p = reqs[i].page;
            let pte_page = unsafe { &mut (*pte_p) };
            let phyaddr = PhysAddr::new(pte_page[reqs[i].index]);

            if phyaddr.bottom_bit() {
                // this page has been prefetched, or at least in the list
                continue;
            }

            // 1. set the page table entry's bottom bit to 1 to prevent future prefetch
            let remote_pa = phyaddr.real_addr();
            // let remote_pa = pte_page[reqs[i].index];

            // FIXME: this code assumes the remote PA never changes for this children
            // To fix this, we need to instrumnet another bits in the address
            pte_page[reqs[i].index] = K_MAGIC_IN_PREFETCH;

            // 2. submit the RDMA request to read the page
            let user_page =
                unsafe { crate::bindings::pmem_alloc_page(crate::bindings::PMEM_GFP_HIGHUSER) };
            let new_page_va = unsafe { crate::bindings::pmem_page_to_virt(user_page) as u64 };

            // TODO: doorbell optimization
            let payload = DCReqPayload::new(
                unsafe { Arc::new(MemoryRegion::new_from_raw(self.conn.get_qp().ctx().clone(), new_page_va as _, 4096).unwrap()) },
                0..4096,
                true,
                RDMAOp::READ,
                self.access_info.rkey,
                remote_pa,
                self.access_info.access_handler.clone(),
            );

            // crate::log::debug!("post reqs {}", self.access_info.dct_num);
            // send the requests
            self.conn.post(&payload).unwrap(); // FIXME: it should never fail

            // 3. record the prefetch information here
            self.pending_queues.push_back(ReplyEntry {
                pt: reqs[i].page,
                idx: reqs[i].index,
                user_page: user_page,
            });
        }
    }
}

impl Future for DCAsyncPrefetcher {
    type Output = *mut page;
    type Error = <DCConn as Future>::Error;

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        match self.conn.poll() {
            Ok(Async::Ready(_wc)) => {
                // must have one
                let v = self.pending_queues.pop_front().unwrap();
                let pte_p = v.pt;
                let pte_page = unsafe { &mut (*pte_p) };

                assert!(PhysAddr::new(v.user_page as _).bottom_bit() == false);
                pte_page[v.idx] =
                    PhysAddr::encode(v.user_page as u64, PhysAddrBitFlag::Prefetch as _);

                return Ok(Async::Ready(v.user_page));
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => return Err(e),
        }
    }
}
