#![no_std]
#![feature(generic_associated_types)]

extern crate alloc;

use alloc::string::String;

use krdma_test::{krdma_test, krdma_drop};

use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util::string::ptr2string;
use rust_kernel_linux_util::timer::KTimer;
use rust_kernel_linux_util as log;

use mitosis_util::bench::*;
use mitosis_util::reporter::*;

use os_network::rdma::ConnMeta;
use os_network::rdma::rc::RCFactoryWPath;
use os_network::rdma::ConnMetaWPath;
use os_network::ConnFactory;

use KRdmaKit::KDriver;
use KRdmaKit::device::RNIC;

use mitosis_macros::{declare_global, declare_module_param};

declare_module_param!(remote_service_id_base, u64);
declare_module_param!(nic_count, u64);
declare_module_param!(running_secs, u32);
declare_module_param!(report_interval, u32);
declare_module_param!(thread_count, u64);
declare_module_param!(gids, *mut u8);

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);
declare_global!(REMOTE_GIDS, alloc::vec::Vec<alloc::string::String>);
declare_global!(REMOTE_NIC_COUNT, u64);

pub struct RCConnBench<'a> {
    factory: RCFactoryWPath<'a>,
    meta: ConnMetaWPath,
}

impl RCConnBench<'_> {
    fn get_local_nic(thread_id: u64) -> &'static RNIC {
        let index = thread_id % nic_count::read();
        unsafe { &KDRIVER::get_ref().devices()[index as usize] }
    }

    fn get_conn_meta(thread_id: u64) -> ConnMeta {
        let index = thread_id % unsafe { *REMOTE_NIC_COUNT::get_ref() };
        let gid = unsafe { REMOTE_GIDS::get_ref()[index as usize].clone() };
        let service_id = remote_service_id_base::read();
        ConnMeta {
            gid: gid,
            service_id: service_id,
            qd_hint: 0,
        }
    }
}

impl<'a> BenchRoutine for RCConnBench<'a> {
    type Args = u64;

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

fn module_main() {
    let mut global_reporter = GlobalReporter::<ConThptReporter>::new();
    let mut bench = Benchmark::<RCConnBench,ConThptReporter>::new();

    for i in 0..thread_count::read() {
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

#[krdma_test(module_main)]
fn ctx_init() {
    unsafe {
        REMOTE_GIDS::init(ptr2string(gids::read()).split(",").map(|x| String::from(x.trim())).collect());
        KDRIVER::init(KDriver::create().unwrap());
        REMOTE_NIC_COUNT::init(REMOTE_GIDS::get_ref().len() as u64);
    }
}

#[krdma_drop]
fn module_drop() {
    unsafe {
        REMOTE_GIDS::drop();
        KDRIVER::drop();
    }
}
