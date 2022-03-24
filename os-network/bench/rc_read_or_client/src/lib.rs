#![no_std]
#![feature(generic_associated_types, get_mut_unchecked)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;

use core::pin::Pin;

use krdma_test::{krdma_drop, krdma_main};

use rust_kernel_linux_util as log;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::timer::KTimer;
use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util::string::ptr2string;

use linux_kernel_module::c_types::*;

use mitosis_util::bench::*;
use mitosis_util::reporter::*;

use os_network::rdma::payload;
use os_network::rdma::ConnMeta;
use os_network::rdma::rc::RCConn;
use os_network::rdma::rc::RCFactory;
use os_network::msg::UDMsg as RMemory;
use os_network::remote_memory::ToPhys;
use os_network::{block_on, Conn, Factory};

use KRdmaKit::KDriver;
use KRdmaKit::device::RNIC;
use KRdmaKit::random::FastRandom;
use KRdmaKit::rust_kernel_rdma_base::*;

use mitosis_macros::{declare_global, declare_module_param};

declare_module_param!(remote_service_id_base, c_uint);
declare_module_param!(nic_count, c_uint); // how many local NICs to use
declare_module_param!(running_secs, c_uint);
declare_module_param!(report_interval, c_uint);
declare_module_param!(thread_count, c_uint);
declare_module_param!(gids, *mut u8);
declare_module_param!(memory_size, c_ulong);
declare_module_param!(or_factor, c_ulong);

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);
declare_global!(REMOTE_GIDS, alloc::vec::Vec<alloc::string::String>);
declare_global!(RCFACTORIES, alloc::vec::Vec<os_network::rdma::rc::RCFactory<'static>>);

type RCReqPayload = payload::Payload<ib_rdma_wr>;

pub struct RCBenchWorker<'a> {
    local_mem: RMemory,
    rkey: u32,
    lkey: u32,
    remote_addr: u64,
    remote_capacity: usize,
    rc: Arc<RCConn<'a>>,
    random: FastRandom,
}

impl RCBenchWorker<'_> {
    fn get_local_nic(thread_id: usize) -> &'static RNIC {
        let index = thread_id % nic_count::read() as usize;
        unsafe { &KDRIVER::get_ref().devices()[index] }
    }

    fn get_my_conn_meta(thread_id: usize) -> ConnMeta {
        let index = thread_id % unsafe { REMOTE_GIDS::get_ref().len() };
        let gid = unsafe { REMOTE_GIDS::get_ref()[index].clone() };
        ConnMeta {
            gid: gid,
            service_id: remote_service_id_base::read() as u64,
            qd_hint: 0,
        }
    }

    #[inline]
    fn get_my_payload(&mut self) -> RCReqPayload {
        let remote_addr = self.remote_addr;
        let range = self.remote_capacity as u64 / memory_size::read();
        let offset = (self.random.get_next() % range) * memory_size::read();
        let remote_addr = remote_addr + offset;
        RCReqPayload::default()
            .set_laddr(unsafe { self.local_mem.to_phys().0 })
            .set_raddr(remote_addr)
            .set_sz(memory_size::read() as usize)
            .set_lkey(self.lkey)
            .set_rkey(self.rkey)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
    }

    #[inline]
    fn get_my_payload_signaled(&mut self) -> RCReqPayload {
        self.get_my_payload()
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
    }
}

impl<'a> BenchRoutine for RCBenchWorker<'a> {
    type Args = usize;

    fn thread_local_init(args: &Self::Args) -> Self {
        let thread_id = *args;
        let factory = unsafe {
            &RCFACTORIES::get_ref()[thread_id as usize]
        };
        let rc = factory.create(Self::get_my_conn_meta(thread_id)).unwrap();
        let rkey = rc.get_qp().get_remote_mr().get_rkey();
        let remote_addr = rc.get_qp().get_remote_mr().get_addr();
        let remote_capacity = rc.get_qp().get_remote_mr().get_capacity();
        Self {
            local_mem: RMemory::new(memory_size::read() as usize, 0),
            rkey: rkey,
            lkey: unsafe { factory.get_context().get_lkey() },
            remote_addr: remote_addr,
            rc: Arc::new(rc),
            random: FastRandom::new(thread_id as u64),
            remote_capacity: remote_capacity as usize,
        }
    }

    fn op(&mut self) -> Result<(), ()> {
        let mut signaled = self.get_my_payload_signaled();
        let mut signaled = unsafe { Pin::new_unchecked(&mut signaled) };
        payload::Payload::<ib_rdma_wr>::finalize(signaled.as_mut());

        let mut unsignaled = self.get_my_payload();
        let mut unsignaled = unsafe { Pin::new_unchecked(&mut unsignaled) };
        payload::Payload::<ib_rdma_wr>::finalize(unsignaled.as_mut());

        let rc = unsafe {
            Arc::get_mut_unchecked(&mut self.rc)
        };
        rc.post(&signaled.as_ref()).map_err(|_| ())?;

        for _ in 1..or_factor::read() {
            rc.post(&unsignaled.as_ref()).map_err(|_| ())?;
        }

        block_on(rc).map_err(|_| ())?;
        Ok(())
    }
}

use alloc::boxed::Box;

fn ctx_init() {
    unsafe {
        REMOTE_GIDS::init(
            ptr2string(gids::read())
                .split(",")
                .map(|x| String::from(x.trim()))
                .collect(),
        );
        KDRIVER::init(KDriver::create().unwrap());
        RCFACTORIES::init(Vec::new());
        for i in 0..thread_count::read() {
            RCFACTORIES::get_mut().push(RCFactory::new(RCBenchWorker::get_local_nic(i as usize)).unwrap());
        }
    }
}

#[krdma_main]
fn module_main() {
    ctx_init();

    let mut global_reporter = GlobalReporter::<ConThptReporter>::new();
    let mut bench = Benchmark::<RCBenchWorker, ConThptReporter>::new();

    for i in 0..thread_count::read() as usize {
        let ctx = Box::new(ThreadLocalCTX::new(
            i,
            ConThptReporter::new(),
            i as usize,
            Some(i as u32),
        ));
        global_reporter.add(ctx.get_reporter());
        bench.spawn(ctx).expect("should succeed");
    }

    let mut timer = KTimer::new();
    for _ in 0..(running_secs::read() / report_interval::read()) {
        kthread::sleep(report_interval::read());
        let count = global_reporter.report() as i64;
        let passed = timer.get_passed_usec();
        let thpt = 1000000 * count / passed;

        timer.reset();
        log::info!(
            "check global reporter states: {}, passed: {}. thpt : {}",
            count,
            passed,
            thpt
        );
    }

    for _ in 0..thread_count::read() {
        bench.stop_one().unwrap();
    }
}

#[krdma_drop]
fn module_drop() {
    unsafe {
        REMOTE_GIDS::drop();
        RCFACTORIES::drop();
        KDRIVER::drop();
    }
}
