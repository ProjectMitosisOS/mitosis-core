use crate::rdma_context::*;

use alloc::vec::Vec;

pub fn start_instance(config: crate::Config) -> core::option::Option<()> {
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

    // TODO: other services

    Some(())
}

pub fn end_instance() {
    unsafe {
        crate::service_rpc::drop();
    };
    end_rdma();
}
