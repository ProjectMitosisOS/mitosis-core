use core::ops::Range;
use alloc::sync::Arc;
use KRdmaKit::{MemoryRegion, DatagramEndpoint};

#[derive(Copy, Clone)]
pub enum RDMAOp {
    READ,
    WRITE,
}

pub trait Signaled {
    fn is_signaled(&self) -> bool;
    fn set_signaled(self) -> Self;
    fn set_unsignaled(self) -> Self;
}

pub trait LocalMR {
    fn set_local_mr(self, mr: Arc<MemoryRegion>) -> Self;
    fn set_local_mr_range(self, range: Range<u64>) -> Self;
    fn get_local_mr(&self) -> Arc<MemoryRegion>;
    fn get_local_mr_range(&self) -> Range<u64>;

}

pub trait RDMAWR {
    fn set_raddr(self, raddr: u64) -> Self;
    fn set_rkey(self, rkey: u32) -> Self;
    fn set_op(self, op: RDMAOp) -> Self;
    fn get_raddr(&self) -> u64;
    fn get_rkey(&self) -> u32;
    fn get_op(&self) -> RDMAOp;
}

pub trait EndPoint {
    fn set_endpoint(self, endpoint: Arc<DatagramEndpoint>) -> Self;
    fn get_endpoint(&self) -> Arc<DatagramEndpoint>;
}

pub mod dc;
pub mod rc;
pub mod ud;
