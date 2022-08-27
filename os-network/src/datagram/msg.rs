use crate::bytes::*;
use alloc::sync::Arc;
use KRdmaKit::memory_region::MemoryRegion;
use KRdmaKit::context::Context;

/// UD must use physical address.
pub struct UDMsg {
    inner: Arc<MemoryRegion>,
    bytes: BytesMut,
    pa: u64,
    imm: u32,
}

impl ToBytes for UDMsg {
    fn get_bytes(&self) -> &BytesMut {
        &self.bytes
    }

    fn get_bytes_mut(&mut self) -> &mut BytesMut {
        &mut self.bytes
    }
}

impl UDMsg {
    /// Allocate memory and create a UDMsg
    /// 
    /// #Argument
    /// * `size` - the memory to be allocated
    /// * `imm` - immediate number in the UD message
    /// * `context` - Context related to the memory region
    pub fn new(size: usize, imm : u32, context: Arc<Context>) -> Self {
        let mr = MemoryRegion::new(context, size)
            .expect("Memory allocation should succeed.");
        let bytes = unsafe { BytesMut::from_raw(mr.get_virt_addr() as *mut u8, size) };
        let pa = unsafe { mr.get_rdma_addr() as u64 };
        Self {
            inner: Arc::new(mr),
            bytes,
            pa,
            imm,
        }
    }

    /// Create a UDMsg from a memory region
    /// 
    /// #Argument
    /// * `MemoryRegion` - the smart pointer of the memory region
    /// * `imm` - immediate number in UD message
    pub fn new_from(mr: Arc<MemoryRegion>, imm: u32) -> Self {
        let bytes = unsafe { BytesMut::from_raw(mr.get_virt_addr() as *mut u8, mr.capacity()) };
        let pa = unsafe { mr.get_rdma_addr() as u64 };
        Self {
            inner: mr,
            bytes,
            pa,
            imm,
        }
    }

    #[inline]
    pub fn get_pa(&self) -> u64 {
        self.pa
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn set_imm(mut self, imm: u32) -> Self {
        self.imm = imm;
        self
    }

    pub fn get_inner(&self) -> Arc<MemoryRegion> {
        self.inner.clone()
    }
}

use core::fmt::{Arguments, Result, Write};

impl Write for UDMsg {
    #[inline]
    fn write_str(&mut self, s: &str) -> Result {
        self.bytes.write_str(s)
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments<'_>) -> Result {
        core::fmt::write(self, args)
    }
}

impl crate::rpc::AllocMsgBuf for UDMsg {
    type Context = Arc<Context>;
    fn create(size: usize, imm: u32, context: Self::Context) -> Self {
        Self::new(size, imm, context)
    }
}
