use alloc::vec::Vec;

/// The pool maintains a thread_local_pool of callers
/// Each CPU core can use the dedicated pool
pub struct CallerPool { 
    // not implemented 
}

impl CallerPool { 
    pub fn new(config : &crate::Config) -> core::option::Option<Self> { 
        Some(Self { })
    }
}