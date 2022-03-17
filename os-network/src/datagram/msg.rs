use crate::bytes::BytesMut;
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
    pub fn to_ud_wr(
        &self,
        addr: &KRdmaKit::cm::EndPoint,
    ) -> crate::rdma::payload::Payload<ib_ud_wr> {
        let res: crate::rdma::payload::Payload<ib_ud_wr> = Default::default();
        res.set_ah(addr)
            .set_laddr(self.pa)
            .set_sz(self.bytes.len())
            .set_opcode(ib_wr_opcode::IB_WR_SEND_WITH_IMM)
            .set_imm_data(self.imm)
    }
}

impl UDMsg {
    pub fn new_from_phy(mut phy: RMemPhy, imm: u32) -> Self {
        let pa = phy.get_pa(0);
        Self {
            pa: pa,
            bytes: unsafe { BytesMut::from_raw(phy.get_ptr() as _, phy.get_sz() as usize) },
            inner: phy,
            imm: imm,
        }
    }

    pub fn new(size: usize, imm : u32) -> Self {
        Self::new_from_phy(RMemPhy::new(size), imm)
    }

    pub fn get_bytes(&self) -> &BytesMut {
        &self.bytes
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
