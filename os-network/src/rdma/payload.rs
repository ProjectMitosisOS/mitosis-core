use KRdmaKit::rust_kernel_rdma_base::*;

pub trait SendWR {
    fn set_opcode(&mut self, opcode: u32);
    fn set_send_flags(&mut self, send_flags: i32);
    fn set_imm_data(&mut self, imm_data: u32);
    fn set_single_sge(&mut self, sge: *const ib_sge);
}

pub struct Payload<T>
where
    T: Default + SendWR
{
    sge: ib_sge,
    wr: T
}

impl<T> Payload<T> 
where
    T: Default + SendWR
{
    fn set_sge(mut self) -> Self {
        self.wr.set_single_sge(unsafe { self.get_sge_ptr() });
        self
    }
    
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

    pub unsafe fn get_sge_ptr(&self) -> *const ib_sge {
        &self.sge as *const _
    }

    pub unsafe fn get_wr_ptr(&self) -> *const T {
        &self.wr as *const T
    }
}

/// Default Payload will be marked with immediate data as 0
impl<T> Default for Payload<T>
where
    T: Default + SendWR
{
    fn default() -> Self {
        Self {
            sge: Default::default(),
            wr: Default::default()
        }
        .set_imm_data(0)
        .set_sge()
    }
}

pub mod rc_payload;
