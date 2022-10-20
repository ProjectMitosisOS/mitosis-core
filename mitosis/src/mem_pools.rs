use crate::KRdmaKit::consts::MAX_KMALLOC_SZ;
use alloc::{vec::Vec, sync::Arc};
use os_network::{msg::UDMsg as RMemory, KRdmaKit::context::Context};

/// Memory pool stores serialization buffers for the descriptor.
/// This is used for speedup large buffer allocations:
/// For 4MB allocation, kernel may take 1ms
pub struct MemPool {
    pool: Vec<RMemory>,
    capacity: usize,
    context: Arc<Context>,
} 

impl MemPool {
    pub fn new(pool_len: usize, context: Arc<Context>) -> Self {
        let mut ret = Self {
            pool: Default::default(),
            capacity: pool_len,
            context: context.clone(),
        };
        ret.fill_up(pool_len, context);
        ret
    }
}

impl MemPool {
    /// Get one buffer. If no one is available, we fill the pool, which may impact the performance. 
    /// We assume that the kernel can always succeed in allocation. If not, we panic. 
    #[inline]
    pub fn pop_one(&mut self) -> RMemory {
        if self.is_empty() {
            self.fill_up(self.capacity, self.context.clone());
        }
        self.pool.pop().unwrap()
    }

    #[inline]
    fn fill_up(&mut self, len: usize, context: Arc<Context>) {
        for _ in 0..len {
            self.pool.push(RMemory::new(MAX_KMALLOC_SZ, 0, context.clone()));
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
