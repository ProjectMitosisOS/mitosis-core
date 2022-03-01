use super::BenchReporter;

/// Throughput reporter for single-thread case
pub struct ThptReporter {
    counter: u64,
}

impl ThptReporter {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl BenchReporter for ThptReporter {
    type States = f64;

    #[inline]
    fn start(&mut self) {}

    #[inline]
    fn end(&mut self) {
        self.counter += 1;
    }

    #[inline]
    fn report(&self) -> f64 {
        self.counter as f64
    }

    #[inline]
    fn reset(&mut self) {
        self.counter = 0;
    }
}

pub struct ConThptReporter {
    prev: u64,
    next: u64,
}

impl ConThptReporter {
    pub fn new() -> Self {
        Self { prev: 0, next: 0 }
    }
}

impl BenchReporter for ConThptReporter {
    type States = u64;

    #[inline]
    fn start(&mut self) {}

    #[inline]
    fn end(&mut self) {
        self.next += 1;
    }

    #[inline]
    fn report(&self) -> Self::States {
        self.next - self.prev
    }

    #[inline]
    fn reset(&mut self) {
        self.prev = self.next
    }
}
