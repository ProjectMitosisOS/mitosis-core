#![no_std]
#![feature(generic_associated_types)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;

use krdma_test::{krdma_drop, krdma_main};

use rust_kernel_linux_util as log;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util::string::ptr2string;
use rust_kernel_linux_util::timer::KTimer;

use linux_kernel_module::c_types::*;

use mitosis_util::bench::*;
use mitosis_util::reporter::*;

use os_network::rdma::dc::DCFactory;
use os_network::Factory;
use os_network::msg::UDMsg as RMemory;
use os_network::remote_memory::rdma::DCRemoteDevice;
use os_network::remote_memory::Device;
use os_network::remote_memory::rdma::DCKeys;

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

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);
declare_global!(REMOTE_GIDS, alloc::vec::Vec<alloc::string::String>);
declare_global!(RCONTEXTS, alloc::vec::Vec<KRdmaKit::device::RContext<'static>>);
declare_global!(
    DCFACTORIES,
    alloc::vec::Vec<os_network::rdma::dc::DCFactory<'static>>
);

static DEFAULT_QD_HINT: u64 = 73;

pub struct DCBenchWorker<'a> {
    endpoint: EndPoint,
    local_mem: RMemory,
    rkey: u32,
    lkey: u32,
    dc: DCRemoteDevice<'a, RMemory>,
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
            dc: DCRemoteDevice::new(Arc::new(dc)),
            random: FastRandom::new(thread_id as u64),
        }
    }

    fn op(&mut self) -> Result<(), ()> {
        let remote_addr = self.endpoint.mr.get_addr();
        let range = self.endpoint.mr.get_capacity() as u64 / memory_size::read();
        let offset = (self.random.get_next() % range) * memory_size::read();
        let remote_addr = remote_addr + offset;
        unsafe {
            self.dc.read(
                &self.endpoint,
                &remote_addr,
                &DCKeys::new(self.lkey, self.rkey, 73),
                &mut self.local_mem,
            )
        }.map_err(|_| ())
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
        RCONTEXTS::init(Vec::new());
        for i in 0..thread_count::read() {
            RCONTEXTS::get_mut().push(DCBenchWorker::get_local_nic(i as usize).open().unwrap());
        }
        for i in 0..thread_count::read() {
            DCFACTORIES::get_mut().push(DCFactory::new(&RCONTEXTS::get_ref()[i as usize]));
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
        RCONTEXTS::drop();
        KDRIVER::drop();
    }
}
