use crate::rdma_context::*;

pub fn start_instance(config: crate::Config) -> core::option::Option<()> {
    start_rdma(&config).expect("fail to create RDMA context");

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
