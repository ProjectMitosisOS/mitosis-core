use alloc::sync::Arc;

use core::pin::Pin;
use core::marker::PhantomData;

use KRdmaKit::cm::EndPoint;

use crate::conn::Conn;
use crate::rdma::dc::DCConn;

pub mod dc;
pub mod rc;
pub use dc::*;
pub use rc::*;

pub struct MemoryKeys {
    lkey: u32,
    rkey: u32,
}
