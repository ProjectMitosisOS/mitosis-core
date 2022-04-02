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

    // Global shadow process service
    unsafe { crate::sp_service::init(crate::shadow_process_service::ShadowProcessService::new()) };

    // TODO: other services

    // establish an RPC to myself
    for i in 0..config.rpc_threads_num {
        let session_id = calculate_session_id(config.machine_id, i, crate::MAX_RPC_THREADS_CNT);
    }

    Some(())
}

pub fn end_instance() {
    crate::log::info!("Stop MITOSIS instance, start cleaning up...");
    unsafe {
        crate::dc_pool_service::drop();
        crate::service_caller_pool::drop();
        crate::service_rpc::drop();
        crate::sp_service::drop();
    };
    end_rdma();
}

/// calculate the session ID of the remote end handler
pub fn calculate_session_id(mac_id: usize, thread_id: usize, max_nthreads: usize) -> usize {
    mac_id * max_nthreads + thread_id
}

use os_network::ud::UDHyperMeta;

/// Connect the RPC session to the remote nodes
///
/// Note: This function should be called after the start_instance call
///
/// TODO: We should wrap a global lock, since this function can called by many applications
///
/// # Arguments
/// * session_id: the session ID to the remote server. It is carefully calculated according to cal_session_id()
pub fn probe_remote_rpc_end(
    session_id: usize,
    connect_info: crate::rpc_service::HandlerConnectInfo,
) -> core::option::Option<()> {
    let len = unsafe { crate::get_rpc_caller_pool_ref().len() };
    for i in 0..len {
        unsafe { crate::get_rpc_caller_pool_mut() }
            .connect_session_at(
                i,
                session_id, // Notice: it is very important to ensure that session ID is unique!
                UDHyperMeta {
                    // the remote machine's RDMA gid. Since we are unit test, we use the local gid
                    gid: os_network::rdma::RawGID::new(connect_info.gid.clone()).unwrap(),
                    service_id: connect_info.service_id,
                    qd_hint: connect_info.qd_hint,
                },
            )
            .expect("failed to connect the endpoint");
    }
    Some(())
}
