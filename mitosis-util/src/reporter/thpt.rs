use rust_kernel_linux_util::timer::KTimer;

use super::BenchReporter; 

pub struct ThptReporter {
    counter: u64,
    timer: KTimer,
}

impl ThptReporter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            timer: KTimer::new()
        }
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
        (self.counter as f64) / self.timer.get_passed_usec() as f64
    }

    #[inline]
    fn reset(&mut self) {
        self.counter = 0;
        self.timer.reset();
    }
}
