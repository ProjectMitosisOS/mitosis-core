use alloc::collections::VecDeque;

use os_network::rdma::dc::DCConn;
use crate::remote_mapping::PageTable;

use crate::bindings::page;

/// This struct is really, really, unsafe
/// Since I currently don't know how to do it right in rust
/// I will come back to this issue later
pub struct ReplyEntry { 
    pt : *mut PageTable,   // page table to update 
    idx : usize, 
    user_page : *mut page, // user page to hold the requests 
}

/// Each DCAsyncPrefetcher has a DCConn 
pub struct DCAsyncPrefetcher {  
    conn: DCConn<'static>,
    pending_queues : VecDeque<ReplyEntry>, 
}