use crate::reporter::BenchReporter;

pub struct ThreadLocalCTX<Arg, R> 
where R : BenchReporter
{
    pub(crate) arg: Arg,    // input specific to this thread
    pub(crate) reporter: R, // reporter to report the benchmark stats
    pub(crate) id: usize,   // thread id
}

impl<Arg,R> ThreadLocalCTX<Arg, R> 
where R : BenchReporter { 
    pub fn new(arg : Arg, r : R, id : usize) -> Self { 
        Self { arg : arg, reporter : r, id : id}
    }
}