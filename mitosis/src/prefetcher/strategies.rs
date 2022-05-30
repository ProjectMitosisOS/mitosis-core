use core::fmt::Debug;

/// A simple prefetcher that will fetch a const N requests
#[derive(Debug, Default)]
pub struct StepPrefetcher<T: Copy + Debug + Default, const N: usize = 2> {
    inner: super::PrefetchRequests<T>,
}

impl<T: Copy + Debug + Default + super::NeedPrefetch, const N: usize> super::Prefetch
    for StepPrefetcher<T, N>
{
    type Item = T;

    fn generate_request<I>(mut self, src: &mut I) -> super::PrefetchRequests<T>
    where
        I: Iterator<Item = T>,
    {
        let mut count = 0;
        while count < N {
            match src.next() {
                Some(v) => {
                    if v.need_prefetch() {
                        self.inner.add(v);
                        count += 1;
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

/// A simple prefetcher that will fetch a const N requests, ignoreing non-valid operation
#[derive(Debug, Default)]
pub struct ConstPrefetcher<T: Copy + Debug + Default, const N: usize = 2> {
    inner: super::PrefetchRequests<T>,
}

impl<T: Copy + Debug + Default + super::NeedPrefetch, const N: usize> super::Prefetch
    for ConstPrefetcher<T, N>
{
    type Item = T;

    fn generate_request<I>(mut self, src: &mut I) -> super::PrefetchRequests<T>
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

impl<T: Copy + Debug + Default, const N: usize> ConstPrefetcher<T, N> {
    pub fn new() -> Self {
        Default::default()
    }
}
