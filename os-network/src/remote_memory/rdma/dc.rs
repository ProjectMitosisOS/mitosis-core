use KRdmaKit::{DatapathError, MemoryRegion};

use KRdmaKit::queue_pairs::endpoint::DatagramEndpoint;

use crate::future::Async;
use crate::rdma::dc::DCConn;
use crate::Future;

pub struct DCRemoteDevice {
    dc: DCConn,
}

#[allow(dead_code)]
pub type DCKeys = super::MemoryKeys;

impl DCRemoteDevice {
    pub fn new(dc: DCConn) -> Self {
        Self { dc: dc }
    }
}

/// Read/Write memory from remote device with physical memory addresss with DC qp
///
/// # Parameters:
/// - `loc`: The endpoint specifying the target node.
/// - `addr`: The remote physical address of the target memory region.
/// - `key`: The remote memory key.
/// - `to`: The local physical address of the local target memory region.
/// - `size`: The size of the target memory region.
///
/// # Errors:
/// - `DatapathError`: There is something wrong in the data path.
///
impl crate::remote_memory::Device for DCRemoteDevice {
    type RemoteMemory = u64;
    type Location = DatagramEndpoint;
    type Key = DCKeys;
    type IOResult = DatapathError;
    type LocalMemory = u64;
    type Size = usize;

    unsafe fn read(
        &mut self,
        loc: &Self::Location,
        addr: &Self::RemoteMemory,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
        size: &Self::Size,
    ) -> Result<(), Self::IOResult> {
        let qp = self.dc.get_qp();
        let vaddr = rust_kernel_linux_util::bindings::bd_phys_to_virt(*to as _);
        let mr = MemoryRegion::new_from_raw(qp.ctx().clone(), vaddr as _, *size).unwrap();
        let range = 0..*size as u64;
        let signaled = true;
        let raddr = addr;
        let rkey = key.rkey;
        qp.post_send_dc_read(loc, &mr, range, signaled, *raddr, rkey)
    }

    unsafe fn write(
        &mut self,
        loc: &Self::Location,
        addr: &Self::RemoteMemory,
        key: &Self::Key,
        to: &mut Self::LocalMemory,
        size: &Self::Size,
    ) -> Result<(), Self::IOResult> {
        let qp = self.dc.get_qp();
        let vaddr = rust_kernel_linux_util::bindings::bd_phys_to_virt(*to as _);
        let mr = MemoryRegion::new_from_raw(qp.ctx().clone(), vaddr as _, *size).unwrap();
        let range = 0..*size as u64;
        let signaled = true;
        let raddr = addr;
        let rkey = key.rkey;
        qp.post_send_dc_write(loc, &mr, range, signaled, *raddr, rkey)
    }
}

/// Poll the completion from the underlying device
///
/// # Errors:
/// - `WCErr`: This error means that the work completion's status is not correct.
/// See <https://www.rdmamojo.com/2013/02/15/ibv_poll_cq/> for the meaning of each error status.
/// - `DatapathError`: This error means that there is something wrong in polling the work completion.
impl Future for DCRemoteDevice {
    type Output = KRdmaKit::rdma_shim::bindings::ib_wc;

    type Error = crate::rdma::Err;

    fn poll<'a>(&'a mut self) -> crate::future::Poll<Self::Output, Self::Error> {
        let res = self.dc.poll();
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
