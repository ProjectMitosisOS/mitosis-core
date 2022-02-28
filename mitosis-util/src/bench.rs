use alloc::format;
use alloc::vec::Vec;
use alloc::string::String;

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};

use rust_kernel_linux_util as log;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::kthread::JoinHandler;
use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util::timer::KTimer;
use rust_kernel_linux_util::linux_kernel_module::c_types::{c_int, c_void};

/// BenchmarkRoutine provide benchmark related methods
pub trait BenchmarkRoutine {
    type Prepare;
    type Input;

    /// Method `prepare` receives the thread parameter as a mut ref
    /// and returns a custom `Prepare` data structure
    /// 
    /// This is called at the beginning of each benchmark thread
    fn prepare(data: &mut Self::Input) -> Self::Prepare;

    /// Method `routine` receives the custom `Prepare` data structure
    /// and returns a result.
    /// 
    /// This is called in the critical path of each benchmark thread
    /// and counted in OP_COUNT.
    fn routine(prepare: &mut Self::Prepare) -> Result<(), ()>;
}

/// BenchmarkReporter provide benchmark related methods
pub trait BenchmarkReporter {

    /// Method `create` is called in the prepare phase
    fn create(&mut self);

    /// Method `start` is called before every execution of benchmark critical path
    fn start(&mut self);

    /// Method `end` is called any every completion of benchmark critical path
    fn end(&mut self);

    /// A custom report function
    fn report(&self) -> String;

    /// A custom clear function
    fn clear(&mut self);
}

pub struct ThptReporter {
    sum: u64,
    timer: KTimer,
}

impl Default for ThptReporter {
    fn default() -> Self {
        Self {
            sum: 0,
            timer: KTimer::new()
        }
    }
}

impl BenchmarkReporter for ThptReporter {
    fn create(&mut self) {
        self.sum = 0;
        self.timer.reset();
    }

    fn start(&mut self) {
        return;
    }

    fn end(&mut self) {
        self.sum += 1;
    }

    fn report(&self) -> String {
        String::from(format!("completed {} actions in {} usec",
                    self.sum, self.timer.get_passed_usec()))
    }

    fn clear(&mut self) {
        self.sum = 0;
        self.timer.reset();
    }
}

#[derive(Default)]
struct ThreadParameter<U, R>
where
    U: Default, R: BenchmarkReporter + Default
{
    parameter: U,
    reporter: R,
    id: u64,
}

/// Define all the essential info for the benchmark
pub struct Benchmark<T, U, R> 
where
    T: BenchmarkRoutine, U: Default, R: BenchmarkReporter + Default
{
    threads: Vec<JoinHandler>,
    thread_parameters: Vec<ThreadParameter<U, R>>,
    thread_count: u64,
    phantom: PhantomData<T>,
}

/// Define the essential functions to
/// create, start and stop the benchmark
impl<T, U, R> Benchmark<T, U, R>
where
    T: BenchmarkRoutine + BenchmarkRoutine<Input = U>, U: Default + Clone, R: BenchmarkReporter + Default
{   
    pub fn new(thread_parameters: Vec<U>) -> Self {
        let thread_count = thread_parameters.len();
        let mut parameters = Vec::new();
        for (pos, p) in thread_parameters.iter().enumerate() {
            let mut parameter = ThreadParameter::<U, R>::default();
            parameter.id = pos as u64;
            parameter.parameter = p.clone();
            parameters.push(parameter);
        }
        Self {
            threads: Vec::new(),
            thread_parameters: parameters,
            thread_count: thread_count as u64,
            phantom: PhantomData
        }
    }
    
    pub fn start(&mut self) -> Result<(), ()> {
        for i in 0..self.thread_count {
            let builder = kthread::Builder::new()
                                        .set_name(format!("Benchmark Thread {}", i))
                                        .set_parameter(&self.thread_parameters[i as usize] as *const _ as *mut c_void);
            let handler = builder.spawn(Benchmark::<T, U, R>::client_thread);
            if handler.is_err() {
                log::error!("spawn thread failed");
                return Err(());
            }
            self.threads.push(handler.unwrap());
        }
        return Ok(());
    }

    pub fn stop(&mut self) -> Result<(), ()> {
        let count = self.threads.len();
        for _i in 0..count {
            let handler = self.threads.pop().unwrap();
            handler.join();
        }
        Ok(())
    }

    pub fn report(&self) -> String {
        let mut result = String::from("");
        for param in self.thread_parameters.iter() {
            result.push_str(&param.reporter.report());
            result.push_str("\n");
        }
        result
    }

    pub fn report_and_clear(&mut self) -> String {
        let result = self.report();
        for param in self.thread_parameters.iter_mut() {
            param.reporter.clear();
        }
        result
    }
}

/// The actual running kthread of benchmark
impl<T, U, R> Benchmark<T, U, R>
where
    T: BenchmarkRoutine + BenchmarkRoutine<Input = U>, U: Default, R: BenchmarkReporter + Default
{
    extern "C" fn client_thread(
        data: *mut c_void,
    ) -> c_int {
        let data = data as *mut ThreadParameter<U, R>;
        let mut prepared = T::prepare(unsafe { &mut (*data).parameter });
        let thread_id = unsafe { (*data).id };
        log::debug!("thread {} starts benchmarking", thread_id);
        unsafe { (*data).reporter.create() };
        while !kthread::should_stop() {
            compiler_fence(Ordering::SeqCst);
            unsafe { (*data).reporter.start() };
            let result = T::routine(&mut prepared);
            if result.is_err() {
                log::error!("error in benchmark routine, wait to exit!");
                while !kthread::should_stop() { kthread::sleep(1) }
                break;
            }
            unsafe { (*data).reporter.end() };
        }
        log::debug!("thread {} ends benchmarking", thread_id);
        0
    }
}
