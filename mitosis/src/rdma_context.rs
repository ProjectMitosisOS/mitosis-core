use alloc::vec::Vec;

use os_network::KRdmaKit;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::*;

use os_network::rpc::*;

const service_id_base: u64 = 73; // not using 0 to prevent error

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
            contexts.push(
                crate::rdma_driver::get_mut()
                    .devices()
                    .get(i)
                    .expect("no available RDMA NIC")
                    .open()
                    .expect("failed to create RDMA context"),
            );
        }
        crate::rdma_contexts::init(contexts);
    };

    unsafe {
        let mut servers = Vec::new();
        for i in 0..config.num_nics_used {
            servers.push(
                RCtrl::create(
                    service_id_base + i as u64,
                    crate::rdma_contexts::get_ref()
                        .get(i)
                        .expect("fatal: cannot get the created context"),
                )
                .expect("failed to create cm server on NIC"),
            )
        }
    };

    Some(())
}

pub fn end_rdma() {
    // Note: the **order** of drop is very important here
    unsafe {
        crate::rdma_cm_service::drop();
        crate::rdma_contexts::drop();
        crate::rdma_driver::drop();
    };
}
