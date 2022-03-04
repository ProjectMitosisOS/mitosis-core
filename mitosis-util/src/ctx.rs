use crate::reporter::BenchReporter;
use alloc::sync::Arc;
use core::cell::UnsafeCell;

pub struct ThreadLocalCTX<Arg, R> 
where R : BenchReporter
{
    pub(crate) arg: Arg,    // input specific to this thread
    pub(crate) reporter: Arc<UnsafeCell<R>>, // reporter to report the benchmark stats
    pub(crate) id: usize,   // thread id
    pub(crate) cpu_id: Option<u32>, // id of the cpu which this thread might be bound to
}

impl<Arg,R> ThreadLocalCTX<Arg, R> 
where R : BenchReporter { 
    pub fn new(arg : Arg, r : R, id : usize, cpu_id : Option<u32>) -> Self { 
        Self { arg : arg, reporter : Arc::new(r.into()), id : id, cpu_id : cpu_id}
    }

    pub fn get_reporter(&self) -> Arc<UnsafeCell<R>> { 
        self.reporter.clone()
    }
}