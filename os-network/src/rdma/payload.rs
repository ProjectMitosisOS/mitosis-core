use KRdmaKit::rust_kernel_rdma_base::*;

pub trait SendWR {
    fn set_opcode(&mut self, opcode: u32);
    fn set_send_flags(&mut self, send_flags: i32);
    fn set_imm_data(&mut self, imm_data: u32);
}

pub struct Payload<T>
where
    T: Default + SendWR + Copy
{
    sge: ib_sge,
    wr: T
}

impl<T> Payload<T> 
where
    T: Default + SendWR + Copy
{
    pub fn set_laddr(mut self, laddr: u64) -> Self {
        self.sge.addr = laddr;
        self
    }

    pub fn set_sz(mut self, sz: usize) -> Self {
        self.sge.length = sz as u32;
        self
    }

    pub fn set_lkey(mut self, lkey: u32) -> Self {
        self.sge.lkey = lkey;
        self
    }

    pub fn set_opcode(mut self, opcode: u32) -> Self {
        self.wr.set_opcode(opcode);
        self
    }

    pub fn set_send_flags(mut self, send_flags: i32) -> Self {
        self.wr.set_send_flags(send_flags);
        self
    }

    pub fn set_imm_data(mut self, imm_data: u32) -> Self {
        self.wr.set_imm_data(imm_data);
        self
    }

    pub fn get_sge(&self) -> ib_sge {
        self.sge
    }

    pub fn get_wr(&self) -> T {
        self.wr
    }
}

/// Default Payload will be marked as SIGNALED and with immediate data as 0
impl<T> Default for Payload<T>
where
    T: Default + SendWR + Copy
{
    fn default() -> Self {
        Self {
            sge: Default::default(),
            wr: Default::default()
        }
        .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
        .set_imm_data(0)
    }
}

pub mod rc_payload;
