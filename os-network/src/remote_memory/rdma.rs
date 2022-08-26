pub mod dc;
pub mod rc;
pub use dc::*;
pub use rc::*;

pub struct MemoryKeys {
    rkey: u32,
}

impl MemoryKeys {
    pub fn new(rkey: u32) -> Self {
        Self {
            rkey: rkey,
        }
    }
}
