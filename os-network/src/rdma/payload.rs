use KRdmaKit::cm::EndPoint;
use KRdmaKit::rust_kernel_rdma_base::*;

pub trait SendWR {
    fn set_opcode(&mut self, opcode: u32);
    fn get_opcode(&self) -> u32;

    fn set_send_flags(&mut self, send_flags: i32);
    fn set_imm_data(&mut self, imm_data: u32);

    fn set_sge_ptr(&mut self, sge: *mut ib_sge);
    fn get_sge_ptr(&self) -> *const ib_sge;
}

pub trait RDMAWR {
    fn set_raddr(&mut self, raddr: u64);
    fn set_rkey(&mut self, rkey: u32);
}

pub trait UDWR {
    fn set_ah(&mut self, end_point: &EndPoint);
}

pub struct Payload<T>
where
    T: Default + SendWR,
{
    wr: T,
    sge: ib_sge,
}

impl<T> Payload<T>
where
    T: Default + SendWR,
{
    pub fn set_my_sge_ptr(&mut self) {
        self.wr.set_sge_ptr(unsafe { self.get_sge_ptr() });
    }

    pub fn print_sge(&self) {
        unsafe { crate::log::debug!("{:?} {:?}", self.wr.get_sge_ptr(), self.get_sge_ptr()) };
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

    pub fn get_opcode(&self) -> u32 {
        self.wr.get_opcode()
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

    pub unsafe fn get_sge_ptr(&self) -> *mut ib_sge {
        &self.sge as *const _ as *mut _
    }

    pub unsafe fn get_wr_ptr(&self) -> *const T {
        &self.wr as *const T
    }
}

impl<T> Payload<T>
where
    T: Default + SendWR + RDMAWR,
{
    pub fn set_raddr(mut self, raddr: u64) -> Self {
        self.wr.set_raddr(raddr);
        self
    }

    pub fn set_rkey(mut self, rkey: u32) -> Self {
        self.wr.set_rkey(rkey);
        self
    }
}

impl<T> Payload<T>
where
    T: Default + SendWR + UDWR,
{
    pub fn set_ah(mut self, endpoint: &EndPoint) -> Self {
        self.wr.set_ah(endpoint);
        self
    }
}

/// Default Payload will be marked with immediate data as 0
impl<T> Default for Payload<T>
where
    T: Default + SendWR,
{
    fn default() -> Self {
        let mut res = Self {
            wr: Default::default(),
            sge: Default::default(),
        };
        res.set_my_sge_ptr();
        res
    }
}

pub mod dc;
pub mod rc;
pub mod ud;
