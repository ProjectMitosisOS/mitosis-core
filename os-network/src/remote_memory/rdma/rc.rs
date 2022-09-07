use alloc::sync::Arc;
use KRdmaKit::{DatapathError, MemoryRegion};

use crate::conn::Conn;
use crate::future::Async;
use crate::rdma::payload::rc::RCReqPayload;
use crate::rdma::payload::RDMAOp;
use crate::rdma::rc::RCConn;
use crate::Future;

pub struct RCRemoteDevice {
    rc:RCConn,
}

pub type RCKeys = super::MemoryKeys;

impl RCRemoteDevice {
    pub fn new(rc:RCConn) -> Self {
        Self { rc: rc }
    }
}

/// Read/Write memory from remote device with physical memory addresss with DC qp
///
/// # Parameters:
/// - `addr`: The remote physical address of the target memory region
/// - `key`: The remote memory key
/// - `to`: The local physical address of the local target memory region
/// - `size`: The size of the target memory region
///
/// # Errors:
/// - `DatapathError`: There is something wrong in the data path.
///
impl crate::remote_memory::Device for RCRemoteDevice {
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
            MemoryRegion::new_from_raw(self.rc.get_qp().ctx().clone(), vaddr as _, *size).unwrap(),
        );
        let range = 0..*size as u64;
        let signaled = true;
        let op = RDMAOp::READ;
        let rkey = key.rkey;
        let raddr = *addr;
        let payload = RCReqPayload::new(mr, range, signaled, op, rkey, raddr);
        self.rc.post(&payload)
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
            MemoryRegion::new_from_raw(self.rc.get_qp().ctx().clone(), vaddr as _, *size).unwrap(),
        );
        let range = 0..*size as u64;
        let signaled = true;
        let op = RDMAOp::WRITE;
        let rkey = key.rkey;
        let raddr = *addr;
        let payload = RCReqPayload::new(mr, range, signaled, op, rkey, raddr);
        self.rc.post(&payload)
    }
}

/// Poll the completion from the underlying device
///
/// # Errors:
/// - `WCErr`: This error means that the work completion's status is not correct.
/// See https://www.rdmamojo.com/2013/02/15/ibv_poll_cq/ for the meaning of each error status.
/// - `DatapathError`: This error means that there is something wrong in polling the work completion.
impl Future for RCRemoteDevice {
    type Output = KRdmaKit::rdma_shim::bindings::ib_wc;

    type Error = crate::rdma::Err;

    fn poll<'a>(&'a mut self) -> crate::future::Poll<Self::Output, Self::Error> {
        let res = self.rc.poll();
        match res {
            Ok(Async::Ready(res)) => {
                if res.status == rust_kernel_rdma_base::ib_wc_status::IB_WC_SUCCESS {
                    Ok(Async::Ready(res))
                } else {
                    Err(crate::rdma::Err::WCErr(unsafe {
                        core::mem::transmute(res.status)
                    }))
                }
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(crate::rdma::Err::DatapathError(e)),
        }
    }
}
