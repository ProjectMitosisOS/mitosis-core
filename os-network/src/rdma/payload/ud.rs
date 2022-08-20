use KRdmaKit::{DatagramEndpoint, MemoryRegion};
use alloc::sync::Arc;

use core::ops::Range;

pub struct UDReqPayload {
    mr: Arc<MemoryRegion>,
    range: Range<u64>,
    signaled: bool,
    endpoint: Arc<DatagramEndpoint>,
}

impl UDReqPayload {
    pub fn new(mr: Arc<MemoryRegion>, range: Range<u64>, signaled: bool, endpoint: Arc<DatagramEndpoint>) -> Self {
        Self { mr, range, signaled, endpoint }
    }
}

impl super::Signaled for UDReqPayload {
    fn is_signaled(&self) -> bool {
        return self.signaled
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

impl super::EndPoint for UDReqPayload {
    fn set_endpoint(mut self, endpoint: Arc<DatagramEndpoint>) -> Self {
        self.endpoint = endpoint;
        self
    }

    fn get_endpoint(&self) -> Arc<DatagramEndpoint> {
        self.endpoint.clone()
    }
}

impl super::LocalMR for UDReqPayload {
    fn set_local_mr(mut self, mr: Arc<KRdmaKit::MemoryRegion>) -> Self {
        self.mr = mr;
        self
    }

    fn set_local_mr_range(mut self, range: Range<u64>) -> Self {
        self.range = range;
        self
    }

    fn get_local_mr(&self) -> Arc<KRdmaKit::MemoryRegion> {
        self.mr.clone()
    }

    fn get_local_mr_range(&self) -> Range<u64> {
        self.range.clone()
    }
}
