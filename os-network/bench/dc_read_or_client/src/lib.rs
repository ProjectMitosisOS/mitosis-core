#![no_std]
#![feature(generic_associated_types, get_mut_unchecked)]

extern crate alloc;

use core::pin::Pin;
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::string::String;

use krdma_test::{krdma_drop, krdma_main};

use rust_kernel_linux_util as log;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::timer::KTimer;
use rust_kernel_linux_util::string::ptr2string;
use rust_kernel_linux_util::linux_kernel_module;

use linux_kernel_module::c_types::*;

use mitosis_util::bench::*;
use mitosis_util::reporter::*;

use os_network::Conn;
use os_network::Factory;
use os_network::block_on;
use os_network::rdma::payload;
use os_network::rdma::dc::DCConn;
use os_network::rdma::dc::DCFactory;
use os_network::remote_memory::ToPhys;
use os_network::msg::UDMsg as RMemory;

use KRdmaKit::KDriver;
use KRdmaKit::device::RNIC;
use KRdmaKit::device::RContext;
use KRdmaKit::random::FastRandom;
use KRdmaKit::cm::{EndPoint, SidrCM};
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
declare_global!(DCFACTORIES, alloc::vec::Vec<os_network::rdma::dc::DCFactory<'static>>);

static DEFAULT_QD_HINT: u64 = 73;

type DCReqPayload = payload::Payload<ib_dc_wr>;

pub struct DCBenchWorker<'a> {
    endpoint: EndPoint,
    local_mem: RMemory,
    rkey: u32,
    lkey: u32,
    dc: Arc<DCConn<'a>>,
    random: FastRandom,
}

impl DCBenchWorker<'_> {
    fn get_local_nic(thread_id: usize) -> &'static RNIC {
        let index = thread_id % nic_count::read() as usize;
        unsafe { &KDRIVER::get_ref().devices()[index] }
    }

    fn get_my_endpoint(ctx: &RContext, thread_id: usize) -> EndPoint {
        let index = thread_id % unsafe { REMOTE_GIDS::get_ref().len() };
        let gid = unsafe { REMOTE_GIDS::get_ref()[index].clone() };
        let service_id = remote_service_id_base::read();
        let path_res = ctx.explore_path(gid, service_id as u64).unwrap();
        let mut sidr_cm = SidrCM::new(ctx, core::ptr::null_mut()).unwrap();
        let remote_info = sidr_cm.sidr_connect(
            path_res, service_id as u64, DEFAULT_QD_HINT);
        remote_info.unwrap()
    }

    #[inline]
    fn get_my_payload(&mut self) -> DCReqPayload {
        let remote_addr = self.endpoint.mr.get_addr();
        let range = self.endpoint.mr.get_capacity() as u64 / memory_size::read();
        let offset = (self.random.get_next() % range) * memory_size::read();
        let remote_addr = remote_addr + offset;
        DCReqPayload::default()
            .set_laddr(unsafe { self.local_mem.to_phys().0 })
            .set_raddr(remote_addr)
            .set_sz(memory_size::read() as usize)
            .set_lkey(self.lkey)
            .set_rkey(self.rkey)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
            .set_ah(&self.endpoint)
    }

    #[inline]
    fn get_my_payload_signaled(&mut self) -> DCReqPayload {
        self.get_my_payload()
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
    }
}

impl<'a> BenchRoutine for DCBenchWorker<'a> {
    type Args = usize;

    fn thread_local_init(args: &Self::Args) -> Self {
        let thread_id = *args;
        let factory = unsafe {
            &DCFACTORIES::get_ref()[thread_id as usize]
        };
        let dc = factory.create(()).unwrap();
        let ctx = factory.get_context();
        let endpoint = Self::get_my_endpoint(ctx, thread_id);
        let rkey = endpoint.mr.get_rkey();
        Self {
            endpoint: endpoint,
            local_mem: RMemory::new(memory_size::read() as usize, 0),
            rkey: rkey,
            lkey: unsafe { factory.get_context().get_lkey() },
            dc: Arc::new(dc),
            random: FastRandom::new(thread_id as u64),
        }
    }

    fn op(&mut self) -> Result<(), ()> {
        let mut signaled = self.get_my_payload_signaled();
        let mut signaled = unsafe { Pin::new_unchecked(&mut signaled) };
        payload::Payload::<ib_dc_wr>::finalize(signaled.as_mut());

        let mut unsignaled = self.get_my_payload();
        let mut unsignaled = unsafe { Pin::new_unchecked(&mut unsignaled) };
        payload::Payload::<ib_dc_wr>::finalize(unsignaled.as_mut());

        let dc = unsafe {
            Arc::get_mut_unchecked(&mut self.dc)
        };
        dc.post(&signaled.as_ref()).map_err(|_| ())?;

        for _ in 1..or_factor::read() {
            dc.post(&unsignaled.as_ref()).map_err(|_| ())?;
        }

        block_on(dc).map_err(|_| ())?;
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
        DCFACTORIES::init(Vec::new());
        for i in 0..thread_count::read() {
            DCFACTORIES::get_mut().push(DCFactory::new(DCBenchWorker::get_local_nic(i as usize)).unwrap());
        }
    }
}

#[krdma_main]
fn module_main() {
    ctx_init();

    let mut global_reporter = GlobalReporter::<ConThptReporter>::new();
    let mut bench = Benchmark::<DCBenchWorker, ConThptReporter>::new();

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
        DCFACTORIES::drop();
        KDRIVER::drop();
    }
}
