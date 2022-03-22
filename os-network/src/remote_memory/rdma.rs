pub mod dc;
pub mod rc;
pub use dc::*;
pub use rc::*;

pub struct MemoryKeys {
    lkey: u32,
    rkey: u32,
}
