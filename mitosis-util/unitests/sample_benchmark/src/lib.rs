#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use krdma_test::krdma_main;

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util as log;

use linux_kernel_module::c_types::c_void;

use mitosis_util::bench::{BenchmarkThreadID, Benchmark, BenchmarkRoutine};

static THREAD_COUNT: u64 = 10;
static TIME: u64 = 10;
static REPORT_INTERVAL: u32 = 1;

// Implement custom `Prepare` data structure

pub struct MyPrepareData {
    thread_id: u64
}

impl MyPrepareData {
    pub fn new(thread_id: u64) -> Self {
        Self {
            thread_id: thread_id
        }
    }
}

impl BenchmarkThreadID for MyPrepareData {
    fn get_thread_id(&self) -> u64 {
        self.thread_id
    } 
}

// Implement custom `BenchmarkRoutine`

pub struct MyBenchmarkRoutine;

impl BenchmarkRoutine for MyBenchmarkRoutine {
    type Prepare = MyPrepareData;

    fn prepare(data: u64) -> Self::Prepare {
            log::info!("prepare in thread {}", data);
            MyPrepareData::new(data)
    }

    fn routine(_prepare: &mut Self::Prepare) -> Result<(), ()> {
            kthread::sleep(1);
            Ok(())
    }
}

#[krdma_main]
fn module_main() {
    let mut parameters = Vec::new();
    let mut sum = 0 as u64;

    for i in 0..THREAD_COUNT {
        parameters.push(i as *mut c_void);
    }

    // Init and start the benchmark with custom routine and required parameters
    let mut benchmark = Benchmark::<MyBenchmarkRoutine>::new(parameters);
    benchmark.start_benchmark().expect("fail to start benchmark");

    // Do some statistics in the main thread
    for _i in 0..TIME {
        kthread::sleep(REPORT_INTERVAL);
        let new_sum = benchmark.report_sum();
        let delta = new_sum - sum;
        sum = new_sum;
        log::info!("complete {} operations in {} second(s)", delta, REPORT_INTERVAL);
    }

    // Stop the benchmark
    benchmark.stop_benchmark().expect("fail to stop benchmark");
    log::info!("total complete {} operations in {} second(s)", sum, TIME);
}
