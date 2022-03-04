#![no_std]

extern crate alloc;

use krdma_test::krdma_test;

use rust_kernel_linux_util as log;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::linux_kernel_module;

use mitosis_util::bench::*;
use mitosis_util::reporter::*;

use mitosis_macros::declare_module_param;

declare_module_param!(nthreads, u32);
declare_module_param!(time, u32);
declare_module_param!(report_interval, u32);

pub struct SampleBench;

impl BenchRoutine for SampleBench {
    type Args = ();

    fn thread_local_init(_args: &Self::Args) -> Self {
        Self
    }

    #[inline]
    fn op(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

use alloc::boxed::Box;

use rust_kernel_linux_util::timer::KTimer;

fn test_global_reporter() {
    log::info!("==test global reporter==");

    let thread_num = crate::nthreads::read() as usize;

    let mut global_reporter = GlobalReporter::<ConThptReporter>::new();
    let mut bench = Benchmark::<SampleBench, ConThptReporter>::new();

    for i in 0..thread_num {
        let ctx = Box::new(ThreadLocalCTX::new((), ConThptReporter::new(), i, None));
        global_reporter.add(ctx.get_reporter());

        bench.spawn(ctx).expect("should succeed");
    }

    let mut timer = KTimer::new();
    // Do some statistics in the main thread
    for _ in 0..time::read() {
        kthread::sleep(report_interval::read());
        let count = global_reporter.report() as i64;

        let passed = timer.get_passed_usec();
        let thpt = count / passed * 1000000; // num / sec
        
        timer.reset();

        log::info!(
            "perf w/o binding: {}, passed: {}. thpt : {} = {}M",
            count,
            passed,
            thpt,
            thpt / 1000000
        );
    }

    for _ in 0..thread_num {
        bench.stop_one().unwrap();
    }
}

fn test_global_reporter_w_cpu_binding() {
    log::info!("==test global reporter with cpu binding==");

    let thread_num = crate::nthreads::read() as usize;

    let mut global_reporter = GlobalReporter::<ConThptReporter>::new();
    let mut bench = Benchmark::<SampleBench, ConThptReporter>::new();

    for i in 0..thread_num {
        let ctx = Box::new(ThreadLocalCTX::new((), ConThptReporter::new(), i, Some(i as u32)));
        global_reporter.add(ctx.get_reporter());

        bench.spawn(ctx).expect("should succeed");
    }

    let mut timer = KTimer::new();
    // Do some statistics in the main thread
    for _ in 0..time::read() {
        kthread::sleep(report_interval::read());
        let count = global_reporter.report() as i64;

        let passed = timer.get_passed_usec();
        let thpt = count / passed * 1000000; // num / sec
        
        timer.reset();

        log::info!(
            "perf w/ binding {}, passed: {}. thpt : {} = {}M",
            count,
            passed,
            thpt,
            thpt / 1000000
        );
    }

    for _ in 0..thread_num {
        bench.stop_one().unwrap();
    }
}

#[krdma_test(test_global_reporter, test_global_reporter_w_cpu_binding)]
fn ctx_init() {}
