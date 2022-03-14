use KRdmaKit::rust_kernel_rdma_base::{ib_dc_wr, ib_sge};
use KRdmaKit::cm::EndPoint;

impl super::SendWR for ib_dc_wr {
    fn set_opcode(&mut self, opcode: u32) {
        self.wr.opcode = opcode;
    }

    fn set_send_flags(&mut self, send_flags: i32) {
        self.wr.send_flags = send_flags;
    }

    fn set_imm_data(&mut self, imm_data: u32) {
        self.wr.ex.imm_data = imm_data;
    }

    fn set_sge_ptr(&mut self, sge: *const ib_sge) {
        self.wr.sg_list = sge as *mut _;
        self.wr.num_sge = 1;
    }
}

impl super::RDMAWR for ib_dc_wr {
    fn set_raddr(&mut self, raddr: u64) {
        self.remote_addr = raddr;
    }
    
    fn set_rkey(&mut self, rkey: u32) {
        self.rkey = rkey;
    }
}

impl super::UDWR for ib_dc_wr {
    fn set_ah(&mut self, end_point: &EndPoint) {
        let ah = end_point.ah;
        let dct_num = end_point.dct_num;

        self.ah = ah;
        self.dct_access_key = 73; // TODO
        self.dct_number = dct_num;
    }    
}
