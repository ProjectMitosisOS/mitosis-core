use KRdmaKit::rust_kernel_rdma_base::{ib_rdma_wr,ib_sge};

impl super::SendWR for ib_rdma_wr {
    fn set_opcode(&mut self, opcode: u32) {
        self.wr.opcode = opcode;
    }

    fn get_opcode(&self) -> u32 { 
        self.wr.opcode
    }    

    fn set_send_flags(&mut self, send_flags: i32) {
        self.wr.send_flags = send_flags;
    }

    fn get_send_flags(&self) -> i32 {
        self.wr.send_flags
    }    

    fn set_imm_data(&mut self, imm_data: u32) {
        self.wr.ex.imm_data = imm_data;
    }

    fn set_sge_ptr(&mut self, sge: *mut ib_sge) {
        self.wr.sg_list = sge;
        self.wr.num_sge = 1;
    }

    fn get_sge_ptr(&self) -> *const ib_sge {
        self.wr.sg_list
    }    
}

impl super::RDMAWR for ib_rdma_wr {
    fn set_raddr(&mut self, raddr: u64) {
        self.remote_addr = raddr;
    }
    
    fn set_rkey(&mut self, rkey: u32) {
        self.rkey = rkey;
    }
}
