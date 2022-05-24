use core::fmt::Debug;

/// The maximum number of pages to prefetch
/// To achieve a better performance, we must restrict it to a small number
pub const K_MAX_PREFETCH_NUM : usize = 4;

#[derive(Debug)]
pub struct PrefetchRequests<T : Copy + Debug + Default> {
    inner : [T; K_MAX_PREFETCH_NUM], 
    sz : usize,
}

impl<T> PrefetchRequests<T> 
where T : Copy + Debug + Default
{
    pub fn new() -> Self { 
        static_assertions::const_assert!(K_MAX_PREFETCH_NUM <= 12);
        Self {
            inner : Default::default(),
            sz : 0
        }
    }

    /// Add an entry to the prefetcher.
    /// If the request has capcaity, then it succeeds. 
    /// Otherwise, it fails silently. 
    /// 
    /// # Return 
    /// - True: add successful
    /// - False: not enough capacity 
    pub fn add(&mut self, data : T) -> bool { 
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

 impl<T> Default for PrefetchRequests<T> 
where T :  Copy + Debug + Default
{ 
    fn default() -> Self {
        Self::new()
    }
}