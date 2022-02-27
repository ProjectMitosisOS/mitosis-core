use alloc::format;
use alloc::vec::Vec;

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};

use rust_kernel_linux_util as log;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::kthread::JoinHandler;
use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util::linux_kernel_module::c_types::{c_int, c_void};

/// OP_COUNT is a private, global vector to record the operation completion count 
static mut OP_COUNT: Option<Vec<u64>> = None;

/// Private function to clear the operation count
fn restore_op_count(sz: usize) {
    let mut op_count_arr = Vec::new();
    for _i in 0..sz {
        op_count_arr.push(0 as u64);
    }
    unsafe {
        OP_COUNT = Some(op_count_arr);
    }
}

/// Private function to sum the operation count
fn sum_op_count() -> u64 {
    unsafe {
        OP_COUNT.as_ref().unwrap().iter().sum()
    }
}

/// BenchmarkThreadID must be implemented to serve as an index into OP_COUNT
pub trait BenchmarkThreadID {
    fn get_thread_id(&self) -> u64;
}

/// BenchmarkRoutine provide benchmark related methods
pub trait BenchmarkRoutine {
    type Prepare: BenchmarkThreadID;

    /// Method `prepare` receives the thread parameter as u64
    /// and returns a custom `Prepare` data structure
    /// 
    /// This is called at the beginning of each benchmark thread
    fn prepare(data: u64) -> Self::Prepare;

    /// Method `routine` receives the custom `Prepare` data structure
    /// and returns a result.
    /// 
    /// This is called in the critical path of each benchmark thread
    /// and counted in OP_COUNT.
    fn routine(prepare: &mut Self::Prepare) -> Result<(), ()>;
}

/// Define all the essential info for the benchmark
pub struct Benchmark<T> 
where
    T: BenchmarkRoutine
{
    threads: Vec<JoinHandler>,
    thread_parameters: Vec<*mut c_void>,
    thread_count: u64,
    phantom: PhantomData<T>,
}

/// Define the essential functions to
/// create, start and stop the benchmark
impl<T> Benchmark<T>
where
    T: BenchmarkRoutine
{   
    pub fn new(thread_parameters: Vec<*mut c_void>) -> Self {
        let thread_count = thread_parameters.len();
        Self {
            threads: Vec::new(),
            thread_parameters: thread_parameters,
            thread_count: thread_count as u64,
            phantom: PhantomData
        }
    }
    
    pub fn start_benchmark(&mut self) -> Result<(), ()> {
        restore_op_count(self.thread_count as usize);
        for i in 0..self.thread_count {
            let builder = kthread::Builder::new()
                                        .set_name(format!("Benchmark Thread {}", i))
                                        .set_parameter(self.thread_parameters[i as usize]);
            let handler = builder.spawn(Benchmark::<T>::client_thread);
            if handler.is_err() {
                log::error!("spawn thread failed");
                return Err(());
            }
            self.threads.push(handler.unwrap());
        }
        return Ok(());
    }

    pub fn report_sum() -> u64 {
        let sum = sum_op_count();
        log::debug!("Total completed operations {}", sum);
        return sum;
    }

    pub fn stop_benchmark(&mut self) -> Result<(), ()> {
        let count = self.threads.len();
        for _i in 0..count {
            let handler = self.threads.pop().unwrap();
            handler.join();
        }
        Ok(())
    }
}

/// The actual running kthread of benchmark
impl<T> Benchmark<T>
where
    T: BenchmarkRoutine
{
    extern "C" fn client_thread(
        data: *mut c_void,
    ) -> c_int {
        let mut prepared = T::prepare(data as u64);
        let thread_id = prepared.get_thread_id();
        log::debug!("thread {} starts benchmarking", thread_id);
        while !kthread::should_stop() {
            compiler_fence(Ordering::SeqCst);
            let result = T::routine(&mut prepared);
            if result.is_err() {
                log::error!("error in benchmark routine, wait to exit!");
                while !kthread::should_stop() { kthread::sleep(1) }
                break;
            }
            unsafe {
                OP_COUNT.as_mut().unwrap()[thread_id as usize] += 1;
            }
        }
        log::debug!("thread {} ends benchmarking", thread_id);
        0
    }
}
