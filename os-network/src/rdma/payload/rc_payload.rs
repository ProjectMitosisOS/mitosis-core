use KRdmaKit::rust_kernel_rdma_base::*;

impl super::SendWR for ib_rdma_wr {
    fn set_opcode(&mut self, opcode: u32) {
        self.wr.opcode = opcode;
    }

    fn set_send_flags(&mut self, send_flags: i32) {
        self.wr.send_flags = send_flags;
    }

    fn set_imm_data(&mut self, imm_data: u32) {
        self.wr.ex.imm_data = imm_data;
    }

    fn set_single_sge(&mut self, sge: *const ib_sge) {
        self.wr.sg_list = sge as *mut _;
        self.wr.num_sge = 1;
    }
}

pub type RCReqPayload = super::Payload<ib_rdma_wr>;

impl RCReqPayload {
    pub fn set_raddr(mut self, raddr: u64) -> Self {
        self.wr.remote_addr = raddr;
        self
    }
    
    pub fn set_rkey(mut self, rkey: u32) -> Self {
        self.wr.rkey = rkey;
        self
    }
}

pub struct RCCompPayload;
