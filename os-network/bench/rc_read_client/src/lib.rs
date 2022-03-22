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

use os_network::rdma::rc::RCFactory;
use os_network::Factory;
use os_network::msg::UDMsg as RMemory;
use os_network::remote_memory::rdma::RCRemoteDevice;
use os_network::remote_memory::Device;
use os_network::remote_memory::rdma::RCKeys;
use os_network::rdma::ConnMeta;

use KRdmaKit::device::RNIC;
use KRdmaKit::KDriver;
use KRdmaKit::device::RContext;
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
declare_global!(RCFACTORIES, alloc::vec::Vec<os_network::rdma::rc::RCFactory<'static>>);

pub struct RCBenchWorker<'a> {
    local_mem: RMemory,
    rkey: u32,
    lkey: u32,
    remote_addr: u64,
    rc: RCRemoteDevice<'a, RMemory>,
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
}

impl<'a> BenchRoutine for RCBenchWorker<'a> {
    type Args = usize;

    fn thread_local_init(args: &Self::Args) -> Self {
        let thread_id = *args;
        let factory = unsafe {
            &RCFACTORIES::get_ref()[thread_id as usize]
        };
        let rc = factory.create(Self::get_my_conn_meta(thread_id)).unwrap();
        let ctx = factory.get_context();
        let rkey = rc.get_qp().get_remote_mr().get_rkey();
        let remote_addr = rc.get_qp().get_remote_mr().get_addr();
        Self {
            local_mem: RMemory::new(memory_size::read() as usize, 0),
            rkey: rkey,
            lkey: unsafe { factory.get_context().get_lkey() },
            remote_addr: remote_addr,
            rc: RCRemoteDevice::new(Arc::new(rc)),
        }
    }

    fn op(&mut self) -> Result<(), ()> {
        unsafe {
            self.rc.read(
                &(),
                &self.remote_addr,
                &RCKeys::new(self.lkey, self.rkey),
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
