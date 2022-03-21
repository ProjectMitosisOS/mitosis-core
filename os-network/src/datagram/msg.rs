use crate::bytes::*;
use KRdmaKit::mem::{Memory, RMemPhy};

/// UD must use physical address.
pub struct UDMsg {
    inner: RMemPhy,
    bytes: BytesMut,
    pa: u64,
    imm: u32,
}

use KRdmaKit::rust_kernel_rdma_base::*;

impl UDMsg {
    /// Transform this UDMsg to raw C data structure - ib_ud_wr
    /// 
    /// #Argument
    /// * `addr` - the endpoint of the target receiver
    pub fn to_ud_wr(
        &self,
        addr: &KRdmaKit::cm::EndPoint,
    ) -> crate::rdma::payload::Payload<ib_ud_wr> {
        self.to_ud_wr_w_resize(addr, self.bytes.len())
    }

    pub fn to_ud_wr_w_resize(
        &self,
        addr: &KRdmaKit::cm::EndPoint,
        sz: usize,
    ) -> crate::rdma::payload::Payload<ib_ud_wr> {
        let res: crate::rdma::payload::Payload<ib_ud_wr> = Default::default();
        res.set_ah(addr)
            .set_laddr(self.pa)
            .set_sz(core::cmp::min(self.bytes.len(), sz))
            .set_opcode(ib_wr_opcode::IB_WR_SEND_WITH_IMM)
            .set_imm_data(self.imm)
    }
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
    /// Create a UDMsg from a kernel physical memory
    /// 
    /// #Argument
    /// * `phy` - allocated physical memory
    /// * `imm` - immediate number in the UD message
    pub fn new_from_phy(mut phy: RMemPhy, imm: u32) -> Self {
        let pa = phy.get_pa(0);
        Self {
            pa: pa,
            bytes: unsafe { BytesMut::from_raw(phy.get_ptr() as _, phy.get_sz() as usize) },
            inner: phy,
            imm: imm,
        }
    }

    /// Allocate memory and create a UDMsg
    /// 
    /// #Argument
    /// * `size` - the memory to be allocated
    /// * `imm` - immediate number in the UD message
    pub fn new(size: usize, imm : u32) -> Self {
        Self::new_from_phy(RMemPhy::new(size), imm)
    }

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

    pub fn to_inner(self) -> RMemPhy {
        self.inner
    }
}

impl crate::remote_memory::ToPhys for UDMsg {
    unsafe fn to_phys(&self) -> (u64, usize) {
        // ugly hack: get_pa requires a mutable reference 
        let inner = unsafe {
            &mut *(&self.inner as *const _ as *mut RMemPhy)
        };
        (inner.get_pa(0), inner.get_sz() as usize)
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
    fn create(size: usize, imm: u32) -> Self {
        Self::new(size, imm)
    }
}
