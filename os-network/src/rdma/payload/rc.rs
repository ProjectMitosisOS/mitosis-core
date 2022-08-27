use core::ops::Range;
use alloc::sync::Arc;
use KRdmaKit::MemoryRegion;

pub struct RCReqPayload {
    mr: Arc<MemoryRegion>,
    range: Range<u64>,
    signaled: bool,
    op: super::RDMAOp,
    rkey: u32,
    raddr: u64,
}

impl RCReqPayload {
    pub fn new(mr: Arc<MemoryRegion>, range: Range<u64>, signaled: bool, op: super::RDMAOp, rkey: u32, raddr: u64) -> Self {
        Self { mr, range, signaled, op, rkey, raddr }
    }
}

impl super::LocalMR for RCReqPayload {
    fn set_local_mr(mut self, mr: Arc<MemoryRegion>) -> Self {
        self.mr = mr;
        self
    }

    fn set_local_mr_range(mut self, range: Range<u64>) -> Self {
        self.range = range;
        self
    }

    fn get_local_mr(&self) -> Arc<MemoryRegion> {
        self.mr.clone()
    }

    fn get_local_mr_range(&self) -> Range<u64> {
        self.range.clone()
    }
}

impl super::Signaled for RCReqPayload {
    fn is_signaled(&self) -> bool {
        self.signaled
    }

    fn set_signaled(mut self) -> Self {
        self.signaled = true;
        self
    }

    fn set_unsignaled(mut self) -> Self {
        self.signaled = false;
        self
    }
}

impl super::RDMAWR for RCReqPayload {
    fn set_raddr(mut self, raddr: u64) -> Self {
        self.raddr = raddr;
        self
    }

    fn set_rkey(mut self, rkey: u32) -> Self {
        self.rkey = rkey;
        self
    }

    fn set_op(mut self, op: super::RDMAOp) -> Self {
        self.op = op;
        self
    }

    fn get_raddr(&self) -> u64 {
        self.raddr
    }

    fn get_rkey(&self) -> u32 {
        self.rkey
    }

    fn get_op(&self) -> super::RDMAOp {
        self.op
    }
}
