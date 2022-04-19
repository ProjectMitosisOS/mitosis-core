use crate::KRdmaKit::consts::MAX_KMALLOC_SZ;
use alloc::vec::Vec;
use os_network::msg::UDMsg as RMemory;

/// Memory pool for descriptor serializations
///
/// T means each pool entry's memory size
pub struct MemPool {
    pool: Vec<RMemory>,
    fill_step: usize
}

impl MemPool {
    pub fn new(pool_len: usize) -> Self {
        let mut res = Self {pool:Default::default(), fill_step:pool_len};
        res.fill_up(pool_len);
        res
    }
}

impl MemPool {
    #[inline]
    pub fn fetch_one_mut(&mut self) -> RMemory {
        if self.is_empty() {
            self.fill_up(self.fill_step);
        }
        self.pool.pop().unwrap()
    }

    #[inline]
    fn fill_up(&mut self, len: usize) {
        for _ in 0..len {
            self.pool.push(RMemory::new(MAX_KMALLOC_SZ, 0));
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.capacity() == 0
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.pool.len()
    }
}
