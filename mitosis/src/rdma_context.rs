use alloc::vec::Vec;

use os_network::KRdmaKit;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::*;

use os_network::rpc::*;

/// Note: need to call `end_rdma` before destroying the kernel module
/// Return
/// * Some(()), the start succeeds
/// * None, fatal error happens, the result is printed
pub fn start_rdma(config: crate::Config) -> core::option::Option<()> {
    unsafe {
        let kdriver = KRdmaKit::KDriver::create()?;
        crate::rdma_driver::init(kdriver);
    };

    unsafe {
        let mut contexts = Vec::new();
        for i in 0..config.num_nics_used {
            contexts.push(crate::rdma_driver::get_mut().devices().get(i)?.open()?);
        }
        crate::rdma_contexts::init(contexts);
    };

    Some(())
}

pub fn end_rdma() {
    // Note: the **order** of drop is very important here
    unsafe {
        crate::rdma_contexts::drop();
        crate::rdma_driver::drop();
    };
}
