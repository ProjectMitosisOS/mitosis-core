use core::fmt::Debug;

/// A simple prefetcher that will fetch a cosnt N requests
#[derive(Debug, Default)]
pub struct StepPrefetcher<T: Copy + Debug + Default, const N: usize = 2> {
    inner: super::PrefetchRequests<T>,
}

impl<T: Copy + Debug + Default + super::NeedPrefetch, const N: usize> StepPrefetcher<T, N> {
    pub fn generate_request<I>(mut self, src: &mut I) -> super::PrefetchRequests<T>
    where
        I: Iterator<Item = T>,
    {
        for _ in 0..N {
            match src.next() {
                Some(v) => {
                    if v.need_prefetch() {
                        self.inner.add(v);
                    }
                }
                None => break,
            }
        }
        self.inner
    }
}

impl<T: Copy + Debug + Default, const N: usize> StepPrefetcher<T, N> {
    pub fn new() -> Self {
        Default::default()
    }
}
