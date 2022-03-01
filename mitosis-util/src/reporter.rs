use alloc::sync::Arc;
use core::cell::UnsafeCell;

/// BenchmarkReporter provide benchmark related methods
pub trait BenchReporter {
    type States;

    /// Method `start` is called before every execution of benchmark critical path
    fn start(&mut self);

    /// Method `end` is called any every completion of benchmark critical path
    fn end(&mut self);

    /// A custom report function
    fn report(&self) -> Self::States;

    /// A custom clear function
    fn reset(&mut self);
}

/// A struct to chain different reporter together
pub struct GlobalReporter<R>
where
    R: BenchReporter,
    R::States: core::ops::Add<Output = R::States> + core::default::Default,
{
    reporters: alloc::vec::Vec<Arc<UnsafeCell<R>>>,
}

impl<R> GlobalReporter<R>
where
    R: BenchReporter,
    R::States: core::ops::Add<Output = R::States> + core::default::Default,
{
    pub fn new() -> Self {
        Self {
            reporters: alloc::vec::Vec::new(),
        }
    }

    pub fn add(&mut self, r: Arc<UnsafeCell<R>>) {
        self.reporters.push(r)
    }

    pub fn report(&self) -> R::States {
        let mut states = core::default::Default::default();
        for r in &self.reporters {
            let reporter : &mut R = unsafe { &mut *r.get() };
            states = states + reporter.report();
            reporter.reset();            
        }
        states
    }
}

pub mod thpt;
pub use thpt::*;
