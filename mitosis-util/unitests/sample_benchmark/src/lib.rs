#![no_std]

extern crate alloc;

use krdma_test::krdma_main;

use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::linux_kernel_module;

use mitosis_util::bench::*;
use mitosis_util::reporter::*;

static THREAD_COUNT: usize = 2;
static TIME: usize = 1;
static REPORT_INTERVAL: u32 = 1;

pub struct SampleBench;

impl BenchRoutine for SampleBench {
    type Args = ();

    fn thread_local_init(_args: &Self::Args) -> Self {
        Self
    }

    fn op(&mut self) -> Result<(), ()> {
        kthread::sleep(1);
        Ok(())
    }
}

use alloc::boxed::Box;

#[krdma_main]
fn module_main() {
    let mut bench = Benchmark::<SampleBench,ThptReporter>::new();
    for i in 0..THREAD_COUNT {
        bench.spawn(Box::new(
            ThreadLocalCTX::new((),ThptReporter::new(), i)
        )).expect("should succeed");
    }

    // Do some statistics in the main thread
    for _ in 0..TIME {
        kthread::sleep(REPORT_INTERVAL);
    }

    for _ in 0..THREAD_COUNT { 
        bench.stop_one().unwrap();
    }
}
