#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use krdma_test::krdma_main;

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util as log;

use mitosis_util::bench::{Benchmark, BenchmarkRoutine, ThptReporter};

static THREAD_COUNT: u64 = 10;
static TIME: u64 = 10;
static REPORT_INTERVAL: u32 = 1;

// Implement custom `Prepare` data structure

pub struct MyPrepareData;

// Implement custom `BenchmarkRoutine`

pub struct MyBenchmarkRoutine;

impl BenchmarkRoutine for MyBenchmarkRoutine {
    type Prepare = MyPrepareData;
    type Input = u64;

    fn prepare(data: &mut Self::Input) -> Self::Prepare {
            log::info!("prepare in thread {}", *data);
            MyPrepareData{}
    }

    fn routine(_prepare: &mut Self::Prepare) -> Result<(), ()> {
            kthread::sleep(1);
            Ok(())
    }
}

#[krdma_main]
fn module_main() {
    let mut parameters = Vec::new();

    for i in 0..THREAD_COUNT {
        parameters.push(i as u64);
    }

    // Init and start the benchmark with custom routine and required parameters
    let mut benchmark = Benchmark::<MyBenchmarkRoutine, u64, ThptReporter>::new(parameters);
    benchmark.start().expect("fail to start benchmark");

    // Do some statistics in the main thread
    for _i in 0..TIME {
        kthread::sleep(REPORT_INTERVAL);
        let report = benchmark.report_and_clear();
        log::info!("{}", report);
    }

    // Stop the benchmark
    benchmark.stop().expect("fail to stop benchmark");
}
