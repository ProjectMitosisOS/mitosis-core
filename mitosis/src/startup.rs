use crate::rdma_context::*;

pub fn start_instance(config : crate::Config) -> core::option::Option<()> { 
    start_rdma(config).expect("fail to create RDMA context");

    // TODO: other services 
    
    Some(())
}

pub fn end_instance() { 
    end_rdma();
}