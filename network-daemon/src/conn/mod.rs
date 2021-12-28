pub mod rc;
pub mod dc;
pub mod ud;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module::KernelResult;

/// RDMAConn is the abstract network connections of mitosis
pub trait RDMAConn {
    // control path
    fn connect(&mut self, addr: &str) -> KernelResult<()>; 

    // data path
    fn one_sided_read(&self, local_addr: u64, remote_addr: u64) -> KernelResult<()>;
    fn one_sided_write(&self, local_addr: u64, remote_addr: u64) -> KernelResult<()>;
    fn send_msg(&self, local_addr: u64) -> KernelResult<()>;
    fn recv_msg(&self, local_addr: u64) -> KernelResult<()>;
}