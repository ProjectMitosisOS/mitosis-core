use crate::rdma_context::*;

#[allow(unused_imports)]
use crate::linux_kernel_module;

use alloc::vec::Vec;

pub fn start_instance(config: crate::Config) -> core::option::Option<()> {
    crate::log::info!("Start MITOSIS instance, init global services");

    start_rdma(&config).expect("fail to create RDMA context");

    // high-level RDMA-related data structures

    // UD factory
    unsafe {
        use os_network::datagram::ud::*;

        let mut ud_factories = Vec::new();
        for c in crate::rdma_contexts::get_ref() {
            ud_factories.push(UDFactory::new(c));
        }
        crate::ud_factories::init(ud_factories);
    };

    // DC factory
    unsafe {
        use os_network::rdma::dc::*;

        let mut dc_factories = Vec::new();
        for c in crate::rdma_contexts::get_ref() {
            dc_factories.push(DCFactory::new(c));
        }
        crate::dc_factories::init(dc_factories);
    };

    // RPC service
    unsafe {
        crate::service_rpc::init(
            crate::rpc_service::Service::new(&config).expect("Failed to create the RPC service. "),
        );
    };

    // RPC caller pool
    unsafe {
        crate::service_caller_pool::init(
            crate::rpc_caller_pool::CallerPool::new(&config)
                .expect("Failed to create the RPC caller pool"),
        )
    };

    // DCQP Pool
    unsafe {
        crate::dc_pool_service::init(
            crate::dc_pool::DCPool::new(&config).expect("Failed to create DCQP pool"),
        )
    };

    // Global parent descriptor pool
    unsafe {
        crate::descriptor_pool::init(
            crate::descriptors::DescriptorFactoryService::create(),
        )
    };

    // TODO: other services

    Some(())
}

pub fn end_instance() {
    crate::log::info!("Stop MITOSIS instance, start cleaning up...");
    unsafe {
        crate::dc_pool_service::drop();
        crate::service_caller_pool::drop();
        crate::service_rpc::drop();
        crate::descriptor_pool::drop();
    };
    end_rdma();
}
