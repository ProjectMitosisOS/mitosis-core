use core::fmt::Display;
use core::ops::Div;
use core::time::Duration;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use hashbrown::HashMap;
use rust_kernel_linux_util::timer::KTimer;

use crate::bindings::{pmem_getnstimeofday, timespec};
use crate::lock_bundler::{BoxedLockBundler, LockBundler};

pub type HistogramId = &'static str;

pub const HISTOGRAM_DUMMY: HistogramId = "dummy";

pub struct HistogramRegistry {
    data: HashMap<HistogramId, Histogram>,
}

impl HistogramRegistry {
    pub fn new() -> Self {
        let mut data = HashMap::new();
        data.insert(HISTOGRAM_DUMMY, Default::default());
        Self { data }
    }

    pub fn scoped_timer(&mut self, id: HistogramId) -> ScopedTimer {
        ScopedTimer::new(self.data.get_mut(id).unwrap())
    }

    // Dump all histogram data by printing.
    pub fn dump(&self) {
        crate::log::info!("Dump histogram data");
        for (name, histogram) in &self.data {
            crate::log::info!("{}", name);
            crate::log::info!("{}", histogram);
        }
    }
}

pub struct Histogram {
    durations: BoxedLockBundler<Vec<Duration>>,
}

impl Default for Histogram {
    fn default() -> Self {
        Self {
            durations: LockBundler::new(Default::default()),
        }
    }
}

impl Histogram {
    pub fn avg(&self) -> Duration {
        self.durations.lock(|durations| {
            if durations.is_empty() {
                Duration::ZERO
            } else {
                durations
                    .iter()
                    .sum::<Duration>()
                    .div(durations.len() as u32)
            }
        })
    }
}

impl Display for Histogram {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "avg: {:?}", self.avg(),)
    }
}

pub struct ScopedTimer<'a> {
    histogram: &'a mut Histogram,
    timer: KTimer,
}

impl<'a> ScopedTimer<'a> {
    pub fn new(histogram: &'a mut Histogram) -> Self {
        Self {
            histogram,
            timer: KTimer::new(),
        }
    }
}

impl<'a> Drop for ScopedTimer<'a> {
    fn drop(&mut self) {
        self.histogram.durations.lock(|durations| {
            durations.push(Duration::from_micros(self.timer.get_passed_usec() as u64))
        });
    }
}
