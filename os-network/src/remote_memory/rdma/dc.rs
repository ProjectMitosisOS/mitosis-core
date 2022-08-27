use KRdmaKit::{DatapathError, MemoryRegion};
use alloc::sync::Arc;

use KRdmaKit::queue_pairs::endpoint::DatagramEndpoint;

use crate::Future;
use crate::future::Async;
use crate::rdma::dc::DCConn;

pub struct DCRemoteDevice {
    dc: Arc<DCConn>,
}

#[allow(dead_code)]
pub type DCKeys = super::MemoryKeys;

impl DCRemoteDevice {
    pub fn new(dc: Arc<DCConn>) -> Self {
        Self {
            dc: dc,
        }
    }
}

/// Read/Write memory from remote device with physical memory with DC qp
impl crate::remote_memory::Device for DCRemoteDevice {
    // remote memory read/write will succeed or return rdma specific error
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
        let mr = MemoryRegion::new_from_raw(
                qp.ctx().clone(),
                vaddr as _,
                *size).unwrap();
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
        let mr = MemoryRegion::new_from_raw(
                qp.ctx().clone(),
                vaddr as _,
                *size).unwrap();
        let range = 0..*size as u64;
        let signaled = true;
        let raddr = addr;
        let rkey = key.rkey;
        qp.post_send_dc_write(loc, &mr, range, signaled, *raddr, rkey)
    }
}

impl Future for DCRemoteDevice {
    type Output = KRdmaKit::rdma_shim::bindings::ib_wc;

    type Error = crate::rdma::Err;

    fn poll<'a>(&'a mut self) -> crate::future::Poll<Self::Output, Self::Error> {
        let res = unsafe {
            Arc::get_mut_unchecked(&mut self.dc)
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
