#![no_std]

extern crate alloc;

use krdma_test::krdma_test;

use rust_kernel_linux_util as log;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::linux_kernel_module;

use mitosis_util::bench::*;
use mitosis_util::reporter::*;

static THREAD_COUNT: usize = 2;
static TIME: usize = 2;
static REPORT_INTERVAL: u32 = 1;

pub struct SampleBench;

impl BenchRoutine for SampleBench {
    type Args = ();

    fn thread_local_init(_args: &Self::Args) -> Self {
        Self
    }

    fn op(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

use alloc::boxed::Box;

fn test_single_reporter() {
    let mut bench = Benchmark::<SampleBench, ThptReporter>::new();
    for i in 0..THREAD_COUNT {
        bench
            .spawn(Box::new(ThreadLocalCTX::new((), ThptReporter::new(), i, None)))
            .expect("should succeed");
    }

    // Do some statistics in the main thread
    for _ in 0..TIME {
        kthread::sleep(REPORT_INTERVAL);
    }

    for _ in 0..THREAD_COUNT {
        bench.stop_one().unwrap();
    }
}

use rust_kernel_linux_util::timer::KTimer;

fn test_global_reporter() {
    log::info!("==test global reporter==");

    let thread_num = THREAD_COUNT * 2;

    let mut global_reporter = GlobalReporter::<ConThptReporter>::new();
    let mut bench = Benchmark::<SampleBench, ConThptReporter>::new();

    for i in 0..thread_num {
        let ctx = Box::new(ThreadLocalCTX::new((), ConThptReporter::new(), i, None));
        global_reporter.add(ctx.get_reporter());

        bench.spawn(ctx).expect("should succeed");
    }

    let mut timer = KTimer::new();
    // Do some statistics in the main thread
    for _ in 0..TIME {
        kthread::sleep(1);
        let count = global_reporter.report() as i64;

        let passed = timer.get_passed_usec();
        let thpt = count / passed * 1000000; // num / sec
        
        timer.reset();

        log::info!(
            "check global reporter states: {}, passed: {}. thpt : {}",
            count,
            passed,
            thpt
        );
    }

    for _ in 0..thread_num {
        bench.stop_one().unwrap();
    }
}

#[krdma_test(test_single_reporter, test_global_reporter)]
fn ctx_init() {}
