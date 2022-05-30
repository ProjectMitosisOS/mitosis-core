use core::fmt::Debug;

use core::ops::{Index, IndexMut};

pub use executor::DCAsyncPrefetcher;
pub use strategies::{ConstPrefetcher, StepPrefetcher};

/// The maximum number of pages to prefetch
/// To achieve a better performance, we must restrict it to a small number
pub const K_MAX_PREFETCH_NUM: usize = 4;

/// Check whether myself needs to be prefetch.
///
/// This trait is used to define when we need prefetch a request.
/// In MITOSIS, we physical address with the 1st bottom bit as 0
/// need to be prefetch.
/// see [`crate::remote_mapping::PhysAddr`],
/// [`crate::remote_mapping::PageEntry`]
pub trait NeedPrefetch {
    /// Return whether myself needs prefetch
    fn need_prefetch(&self) -> bool;
}

/// Generate prefetch request given an iterator
pub trait Prefetch {
    type Item;

    fn generate_request<I>(self, src: &mut I) -> PrefetchRequests<Self::Item>
    where
        I: Iterator<Item = Self::Item>, 
        Self::Item : Copy + Debug + Default;
}

#[derive(Debug)]
pub struct PrefetchRequests<T: Copy + Debug + Default> {
    inner: [T; K_MAX_PREFETCH_NUM],
    sz: usize,
}

impl<T> PrefetchRequests<T>
where
    T: Copy + Debug + Default,
{
    pub fn new() -> Self {
        static_assertions::const_assert!(K_MAX_PREFETCH_NUM <= 12);
        Self {
            inner: Default::default(),
            sz: 0,
        }
    }

    /// Return the number of valid entries
    pub fn len(&self) -> usize {
        self.sz
    }

    /// Add an entry to the prefetcher.
    /// If the request has capcaity, then it succeeds.
    /// Otherwise, it fails silently.
    ///
    /// # Return
    /// - True: add successful
    /// - False: not enough capacity
    pub fn add(&mut self, data: T) -> bool {
        self.inner[self.sz] = data;
        self.sz += 1;
        self.sz <= K_MAX_PREFETCH_NUM
    }

    /// Returns an iterator over the entries of requests
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }
}

impl<T> Index<usize> for PrefetchRequests<T>
where
    T: Copy + Debug + Default,
{
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T> IndexMut<usize> for PrefetchRequests<T>
where
    T: Copy + Debug + Default,
{
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<T> Default for PrefetchRequests<T>
where
    T: Copy + Debug + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

pub mod executor;
pub mod strategies;
