pub struct RPCAnalysis {
    ncalls : usize
}

impl RPCAnalysis { 
    pub fn new() -> Self { 
        Self { 
            ncalls : 0
        }
    }

    pub fn get_ncalls(&self) -> usize { 
        self.ncalls
    }

    #[inline]
    pub fn handle_one(&mut self)  { 
        self.ncalls += 1;
    }
}