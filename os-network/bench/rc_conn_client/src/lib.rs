#![no_std]
#![feature(generic_associated_types)]

extern crate alloc;

use alloc::string::String;

use krdma_test::{krdma_drop, krdma_main};

use rust_kernel_linux_util as log;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util::string::ptr2string;
use rust_kernel_linux_util::timer::KTimer;

use linux_kernel_module::c_types::*;

use mitosis_util::bench::*;
use mitosis_util::reporter::*;

use os_network::rdma::rc::RCFactoryWPath;
use os_network::rdma::{ConnMeta, ConnMetaWPath};
use os_network::ConnFactory;

use KRdmaKit::device::RNIC;
use KRdmaKit::KDriver;

use mitosis_macros::{declare_global, declare_module_param};

declare_module_param!(remote_service_id_base, u64);
declare_module_param!(nic_count, c_uint); // how many local NICs to use
declare_module_param!(running_secs, c_uint);
declare_module_param!(report_interval, c_uint);
declare_module_param!(thread_count, c_uint);
declare_module_param!(gids, *mut u8);

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);
declare_global!(REMOTE_GIDS, alloc::vec::Vec<alloc::string::String>);

pub struct RCConnBenchWorker<'a> {
    factory: RCFactoryWPath<'a>,
    meta: ConnMetaWPath,
}

impl RCConnBenchWorker<'_> {
    fn get_local_nic(thread_id: usize) -> &'static RNIC {
        let index = thread_id % nic_count::read() as usize;
        unsafe { &KDRIVER::get_ref().devices()[index] }
    }

    fn get_conn_meta(thread_id: usize) -> ConnMeta {
        let index = thread_id % unsafe { REMOTE_GIDS::get_ref().len() };
        let gid = unsafe { REMOTE_GIDS::get_ref()[index].clone() };
        let service_id = remote_service_id_base::read();
        ConnMeta {
            gid: gid,
            service_id: service_id,
            qd_hint: 0,
        }
    }
}

impl<'a> BenchRoutine for RCConnBenchWorker<'a> {
    type Args = usize;

    fn thread_local_init(args: &Self::Args) -> Self {
        let thread_id = *args;
        let local_nic = Self::get_local_nic(thread_id);
        let conn_meta = Self::get_conn_meta(thread_id);
        let factory = RCFactoryWPath::new(local_nic).unwrap();
        let conn_meta = factory.convert_meta(conn_meta).unwrap();
        Self {
            factory: factory,
            meta: conn_meta,
        }
    }

    fn op(&mut self) -> Result<(), ()> {
        let mut rc = self.factory.create(self.meta).unwrap();
        if rc.get_status().is_some() {
            return Ok(());
        } else {
            log::error!("failed to create connection");
            return Err(());
        }
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
    }
}

#[krdma_main]
fn module_main() {
    ctx_init();

    let mut global_reporter = GlobalReporter::<ConThptReporter>::new();
    let mut bench = Benchmark::<RCConnBenchWorker, ConThptReporter>::new();

    for i in 0..thread_count::read() as usize {
        let ctx = Box::new(ThreadLocalCTX::new(i, ConThptReporter::new(), i as usize));
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
        KDRIVER::drop();
    }
}
