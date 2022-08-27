use KRdmaKit::{DatapathError, MemoryRegion};
use alloc::sync::Arc;

use crate::future::Async;
use crate::rdma::payload::RDMAOp;
use crate::rdma::payload::rc::RCReqPayload;
use crate::Future;
use crate::conn::Conn;
use crate::rdma::rc::RCConn;

pub struct RCRemoteDevice {
    rc: Arc<RCConn>,
}

pub type RCKeys = super::MemoryKeys;

impl RCRemoteDevice {
    pub fn new(rc: Arc<RCConn>) -> Self {
        Self {
            rc: rc,
        }
    }
}

/// Read/Write memory from remote device with physical memory with RC qp
impl crate::remote_memory::Device for RCRemoteDevice {
    // remote memory read/write will succeed or return rdma specific error
    type RemoteMemory = u64;
    type Location = ();
    type Key = RCKeys;
    type IOResult = DatapathError;
    type LocalMemory = u64;
    type Size = usize;

    unsafe fn read(
        &mut self,
        _loc: &Self::Location,
        addr: &Self::RemoteMemory,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
        size: &Self::Size,
    ) -> Result<(), Self::IOResult> {
        let vaddr = rust_kernel_linux_util::bindings::bd_phys_to_virt(*to as _);
        let mr = Arc::new(
                MemoryRegion::new_from_raw(
                    self.rc.get_qp().ctx().clone(),
                    vaddr as _,
                    *size).unwrap()
            );
        let range = 0..*size as u64;
        let signaled = true;
        let op = RDMAOp::READ;
        let rkey = key.rkey;
        let raddr = *addr;
        let payload = RCReqPayload::new(mr, range, signaled, op, rkey, raddr);
        Arc::get_mut_unchecked(&mut self.rc).post(&payload)
    }

    unsafe fn write(
        &mut self,
        _loc: &Self::Location,
        addr: &Self::RemoteMemory,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
        size: &Self::Size,
    ) -> Result<(), Self::IOResult> {
        let vaddr = rust_kernel_linux_util::bindings::bd_phys_to_virt(*to as _);
        let mr = Arc::new(
                MemoryRegion::new_from_raw(self.rc.get_qp().ctx().clone(),
                vaddr as _,
                *size).unwrap()
            );
        let range = 0..*size as u64;
        let signaled = true;
        let op = RDMAOp::WRITE;
        let rkey = key.rkey;
        let raddr = *addr;
        let payload = RCReqPayload::new(mr, range, signaled, op, rkey, raddr);
        Arc::get_mut_unchecked(&mut self.rc).post(&payload)
    }
}

impl Future for RCRemoteDevice {
    type Output = KRdmaKit::rdma_shim::bindings::ib_wc;

    type Error = crate::rdma::Err;

    fn poll<'a>(&'a mut self) -> crate::future::Poll<Self::Output, Self::Error> {
        let res = unsafe {
            Arc::get_mut_unchecked(&mut self.rc)
        }.poll();
        match res {
            Ok(Async::Ready(res)) => {
                if res.status == rust_kernel_rdma_base::ib_wc_status::IB_WC_SUCCESS {
                    Ok(Async::Ready(res))
                } else {
                    Err(crate::rdma::Err::WCErr(unsafe {
                        core::mem::transmute(res.status)
                    }))
                }
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(crate::rdma::Err::DatapathError(e)),
        }
    }
}
