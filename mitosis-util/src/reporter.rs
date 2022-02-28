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

pub mod thpt;
pub use thpt::ThptReporter;