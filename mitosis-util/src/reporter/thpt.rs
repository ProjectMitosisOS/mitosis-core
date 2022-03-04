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

    #[inline(always)]
    fn start(&mut self) {}

    #[inline(always)]
    fn end(&mut self) {
        self.counter += 1;
    }

    #[inline(always)]
    fn report(&self) -> f64 {
        self.counter as f64
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.counter = 0;
    }
}

#[repr(align(128))]
struct CachedAlignedu64(u64); 

pub struct ConThptReporter {
    prev: CachedAlignedu64,
    next: CachedAlignedu64,
}

impl ConThptReporter {
    pub fn new() -> Self {
        Self { prev: CachedAlignedu64(0), next: CachedAlignedu64(0) }
    }
}

impl BenchReporter for ConThptReporter {
    type States = u64;

    #[inline(always)]
    fn start(&mut self) {}

    #[inline(always)]
    fn end(&mut self) {
        self.next.0 += 1;
    }

    #[inline(always)]
    fn report(&self) -> Self::States {
        self.next.0 - self.prev.0
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.prev.0 = self.next.0
    }
}
