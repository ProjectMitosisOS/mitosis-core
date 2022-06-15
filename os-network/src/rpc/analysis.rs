use hashbrown::HashMap;

#[derive(Debug)]
pub struct RPCAnalysis {
    ncalls: usize,
    pub(crate) session_counts: HashMap<usize, usize>,
}

impl RPCAnalysis {
    pub fn new() -> Self {
        Self {
            ncalls: 0,
            session_counts: Default::default(),
        }
    }

    pub fn get_ncalls(&self) -> usize {
        self.ncalls
    }

    #[inline]
    pub fn handle_one(&mut self) {
        self.ncalls += 1;
    }

    #[inline]
    pub fn handle_session_call(&mut self, id: usize) {
        if self.session_counts.contains_key(&id) {
            self.session_counts.insert(id, self.session_counts[&id] + 1);
        } else {
            self.session_counts.insert(id, 1);
        }
    }
}
