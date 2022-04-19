use crate::KRdmaKit::consts::MAX_KMALLOC_SZ;
use alloc::vec::Vec;
use os_network::msg::UDMsg as RMemory;

/// Memory pool for descriptor serializations
///
/// T means each pool entry's memory size
pub struct MemPool {
    pool: Vec<RMemory>,
    cursor: usize,
    capacity: usize,
}

impl MemPool {
    pub fn new(pool_len: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_len);
        for _ in 0..pool_len {
            pool.push(RMemory::new(MAX_KMALLOC_SZ, 0));
        }
        Self {
            pool,
            cursor: 0,
            capacity: pool_len,
        }
    }
}

impl MemPool {
    #[inline]
    pub fn fetch_one_mut(&mut self) -> &mut RMemory {
        let res = self.pool.get_mut(self.cursor);
        self.cursor = (self.cursor + 1) % self.capacity;
        res.unwrap()
    }
}
