use alloc::boxed::Box;
use alloc::vec::Vec;

use core::sync::atomic::{compiler_fence, Ordering};

use rust_kernel_linux_util as log;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::kthread::JoinHandler;
use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util::linux_kernel_module::c_types::{c_int, c_void};

use crate::reporter::BenchReporter;

/// BenchmarkRoutine provide benchmark related methods
pub trait BenchRoutine {
    type Args;

    /// This is called at the beginning of each benchmark thread
    fn thread_local_init(args: &Self::Args) -> Self;

    /// Method `op` receives the custom `Prepare` data structure
    /// and returns a result.
    ///
    /// This is called in the critical path of each benchmark thread
    /// and recorded by reporter.
    fn op(&mut self) -> Result<(), ()>;
}

/// Define all the essential info for the benchmark
pub struct Benchmark<B, R>
where
    B: BenchRoutine,
    R: BenchReporter,
{
    threads: Vec<JoinHandler>,
    phantom: core::marker::PhantomData<B>,
    phantom_1: core::marker::PhantomData<R>,
}

pub use crate::ctx::ThreadLocalCTX;

/// Define the essential functions to
/// create, start and stop the benchmark
impl<B, R> Benchmark<B, R>
where
    B: BenchRoutine,
    R: BenchReporter,
{
    pub fn new() -> Self {
        Self {
            threads: Vec::new(),
            phantom: core::marker::PhantomData,
            phantom_1: core::marker::PhantomData,
        }
    }

    pub fn spawn(
        &mut self,
        ctx: Box<ThreadLocalCTX<B::Args, R>>,
    ) -> linux_kernel_module::KernelResult<()>
    where
        R: BenchReporter,
    {
        let id = ctx.id;
        let ctx_ptr = Box::into_raw(ctx);

        let builder = kthread::Builder::new()
            .set_name(alloc::format!("Benchmark Thread {}", id))
            .set_parameter(ctx_ptr as *mut c_void);
        let handler = builder.spawn(Self::worker)?;
        self.threads.push(handler);

        Ok(())
    }

    pub fn stop_one(&mut self) -> core::option::Option<()> {
        self.threads.pop().map(|handler| {
            handler.join();
            ()
        })
    }
}

/// The actual running kthread of benchmark
impl<B, R> Benchmark<B, R>
where
    B: BenchRoutine,
    R: BenchReporter,
{
    extern "C" fn worker(ctx: *mut c_void) -> c_int {
        let ctx = unsafe { Box::from_raw(ctx as *mut ThreadLocalCTX<B::Args, R>) };

        log::info!("Bench thread {} started", ctx.id);
        let mut bench = B::thread_local_init(&ctx.arg);

        while !kthread::should_stop() {
            compiler_fence(Ordering::SeqCst);
            unsafe { (*ctx.reporter.get()).start() };
            let result = bench.op();
            if result.is_err() {
                log::error!("error in benchmark routine, wait to exit!");
                while !kthread::should_stop() {
                    kthread::sleep(1)
                }
                break;
            }
            unsafe { (*ctx.reporter.get()).end() };
        }

        log::info!("Bench thread {} finished", ctx.id);
        0
    }
}
