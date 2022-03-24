#![no_std]
#![feature(generic_associated_types, core_intrinsics)]

extern crate alloc;

pub mod ctx;
pub mod reporter;

/// A utility for mass parallel benchmark on single machine
///
/// ## Usage
///
/// 1. Implement BenchRoutine, where `Args` is any customed type
///
/// ```
/// impl BenchRoutine for SampleBench {
///     type Args = ();
///     
///     fn thread_local_init(_args: &Self::Args) -> Self {
///         Self
///     }
///     
///     #[inline]
///     fn op(&mut self) -> Result<(), ()> {
///         Ok(())
///     }
/// }
/// ```
///
/// 2. Create a global reporter with the required reporter type.
///
/// ```
/// let mut global_reporter = GlobalReporter::<ConThptReporter>::new();
/// ```
///
/// 3. Create a benchmark executor with the required reporter type and custom benchmark routine
///
/// ```
/// let mut bench = Benchmark::<SampleBench, ConThptReporter>::new();
/// ```
///
/// 4. Create ThreadLocalCTX for each worker thread and spawn the work thread with Benchmark::spawn
///
/// ```
/// let ctx = Box::new(ThreadLocalCTX::new(
///     ()                      /* Args in BenchRoutine */,
///     ConThptReporter::new()  /* Thread-local Repoter */,
///     i                       /* Thread id */,
///     None                    /* Use Some(cpu_id as u32) to bind worker thread to cpu */ ));
/// global_reporter.add(ctx.get_reporter()); // remember to attach the thread local reporter to global
/// bench.spawn(ctx).expect("should succeed");
/// ```
///
/// 5. Use GlobalReporter::report to report the result
///
/// ```
/// let count = global_reporter.report() as i64;
/// ```
///
/// 6. Use Benchmark::stop_one to stop one worker thread
///
/// ```
/// bench.stop_one().unwrap();
/// ```
///
pub mod bench;

use crate::alloc::string::ToString;

pub fn pretty_print_int(i: i64) -> alloc::string::String {
    let mut s = alloc::string::String::new();
    let i_str = i.to_string();
    let a = i_str.chars().rev().enumerate();
    for (idx, val) in a {
        if idx != 0 && idx % 3 == 0 {
            s.insert(0, ',');
        }
        s.insert(0, val);
    }
    s
}
