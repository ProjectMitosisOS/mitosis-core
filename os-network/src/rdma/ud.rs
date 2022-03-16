// TODO: should move ot the datagram folder
use alloc::sync::Arc;
use alloc::collections::VecDeque;

use crate::Future;
use crate::bytes::RMemRegion;
use crate::future::{Poll,Async};
use crate::bytes::BytesMut;
use crate::Datagram;

use core::marker::PhantomData;
use core::option::Option;

use KRdmaKit::device::{RContext, RNIC};
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::qp::UD;
use KRdmaKit::qp::UDOp;
use KRdmaKit::cm::EndPoint;
use KRdmaKit::mem::RMemPhy;
use KRdmaKit::mem::Memory;

use rust_kernel_linux_util as log;

pub struct UDFactory<'a> {
    rctx: RContext<'a>,
}

impl<'a> UDFactory<'a> {
    pub fn new(hca: &'a RNIC) -> Option<Self> {
        RContext::create(hca).map(|c| Self { rctx: c })
    }

    pub fn get_context(&self) -> &RContext<'_> {
        &self.rctx
    }
}

pub struct UDDatagram<'a, const ENTRY_COUNT: usize, const ENTRY_SIZE: usize> {
    ud: Arc<UD>,
    phantom: PhantomData<&'a ()>,
    queue: VecDeque<RMemRegion>,
    inner_mem: Option<RMemPhy>,
}

impl<const ENTRY_COUNT: usize, const ENTRY_SIZE: usize> UDDatagram<'_, ENTRY_COUNT, ENTRY_SIZE> {
    pub fn get_qp(&self) -> Arc<UD> {
        self.ud.clone()
    }
}

impl<const ENTRY_COUNT: usize, const ENTRY_SIZE: usize> Future for UDDatagram<'_, ENTRY_COUNT, ENTRY_SIZE> {
    type Output = RMemRegion;
    type Error = super::Err;

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        let mut wc: ib_wc = Default::default();
        let cq = self.ud.get_recv_cq();
        let ret = unsafe { bd_ib_poll_cq(cq, 1, &mut wc) };
        match ret {
            0 => {
                return Ok(Async::NotReady);
            },
            1 => {
                if wc.status != ib_wc_status::IB_WC_SUCCESS {
                    return Err(super::Err::Other);
                } else {
                    let memory = self.queue.pop_front();
                    if memory.is_none() {
                        log::error!("internal queue is empty");
                        return Err(super::Err::Other);
                    }
                    return Ok(Async::Ready(memory.unwrap()));
                }
            },
            _ => {
                log::error!("ib_poll_cq returns {}", ret);
                return Err(super::Err::Other);
            },
        }
    }
}

impl<const ENTRY_COUNT: usize, const ENTRY_SIZE: usize> crate::Datagram for UDDatagram<'_, ENTRY_COUNT, ENTRY_SIZE> {
    type AddressHandler = EndPoint;
    type MemoryRegion = RMemRegion;

    fn post_msg(
        &mut self,
        addr: &Self::AddressHandler,
        msg: &Self::MemoryRegion,
    ) -> Result<(), Self::IOResult> {
        let mut op = UDOp::new(&self.ud);
        let res = op.send(msg.get_paddr(), msg.get_lkey(), &addr, msg.get_bytes().len());
        if res.is_err() {
            log::error!("unable to send message");
            return Err(super::Err::Other);
        }
        Ok(())
    }

    fn post_recv_buf(&mut self, buf: Self::MemoryRegion) -> Result<(), Self::IOResult> {
        let mut op = UDOp::new(&self.ud);
        let res = op.post_recv(buf.get_paddr(), buf.get_lkey(), buf.get_bytes().len());
        if res.is_err() {
            log::error!("unable to post recv buffer");
            return Err(super::Err::Other);
        }
        self.queue.push_back(buf);
        Ok(())
    }
}

impl UDFactory<'_> {
    pub fn create<'a, const ENTRY_COUNT: usize, const ENTRY_SIZE: usize>(&self) -> Result<UDDatagram<'a, ENTRY_COUNT, ENTRY_SIZE>, super::Err>
        where Self: 'a
    {
        let ud = UD::new(&self.rctx).ok_or(super::Err::Other)?;
        if ENTRY_COUNT == 0 || ENTRY_SIZE == 0 {
            return Ok(UDDatagram::<'a, ENTRY_COUNT, ENTRY_SIZE> {
                ud: ud,
                phantom: PhantomData,
                queue: VecDeque::new(),
                inner_mem: None,
            });
        }

        // create inner memory for the UD
        let mut inner_mem = RMemPhy::new(ENTRY_COUNT * ENTRY_SIZE);
        let paddr = inner_mem.get_pa(0);
        let vaddr = inner_mem.get_ptr() as *mut u8;
        let mut datagram = UDDatagram::<'a, ENTRY_COUNT, ENTRY_SIZE> {
            ud: ud,
            phantom: PhantomData,
            queue: VecDeque::new(),
            inner_mem: Some(inner_mem),
        };

        // post recv the inner memory
        for i in 0..ENTRY_COUNT {
            let buf = unsafe {
                let vaddr = vaddr.add(i * ENTRY_SIZE);
                let paddr = paddr + (i * ENTRY_SIZE) as u64;
                RMemRegion::new(BytesMut::from_raw(vaddr, ENTRY_SIZE), paddr, self.rctx.get_lkey())
            };
            datagram.post_recv_buf(buf).map_err(|_x| {
                log::error!("unable to post recv buf");
                super::Err::Other
            })?;
        }
        Ok(datagram)
    }
}
