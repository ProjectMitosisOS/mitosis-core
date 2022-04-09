use crate::rdma_context::*;

#[allow(unused_imports)]
use crate::linux_kernel_module;

use rust_kernel_linux_util::timer::KTimer;

use alloc::vec::Vec;

pub fn start_instance(config: crate::Config) -> core::option::Option<()> {
    crate::log::info!("Try to start MITOSIS instance, init global services");
    unsafe {
        crate::mac_id::init(config.machine_id);
        crate::max_caller_num::init(config.max_core_cnt);
        crate::max_nics_used::init(config.num_nics_used); 
        crate::max_cluster_size::init(config.max_cluster_size);
    };

    let timer = KTimer::new();

    start_rdma(&config).expect("fail to create RDMA context");
    crate::log::info!("Initialize RDMA context done");

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
    crate::log::info!("RPC service initializes done");

    // RPC caller pool
    unsafe {
        crate::service_caller_pool::init(
            crate::rpc_caller_pool::CallerPool::new(&config)
                .expect("Failed to create the RPC caller pool"),
        )
    };

    // DCQP & target pool
    unsafe {
        crate::dc_pool_service::init(
            crate::dc_pool::DCPool::new(&config).expect("Failed to create DCQP pool"),
        );
        crate::dc_target_service::init(
            crate::dc_pool::DCTargetPool::new(&config).expect("Failed to create DC Target pool"),
        )
    };

    // Global shadow process service
    unsafe { crate::sp_service::init(crate::shadow_process_service::ShadowProcessService::new()) };

    // TODO: other services

    crate::log::info!("Start waiting for the RPC servers to start...");
    crate::rpc_service::wait_handlers_ready_barrier(config.rpc_threads_num);
    crate::log::info!("All RPC thread handlers initialized!");

    // establish an RPC to myself
    for i in 0..config.rpc_threads_num {
        probe_remote_rpc_end(
            config.machine_id  + config.max_cluster_size * i,
            unsafe { crate::service_rpc::get_ref() }
                .get_connect_info(i)
                .expect("Self RPC handler connection info uninitialized"),
        )
        .expect("failed to connect to my RPC handlers!");
    }
    crate::log::info!("Probe myself RPC handlers done");

    crate::log::info!(
        "All initialization done, takes {} ms",
        timer.get_passed_usec() / 1000
    );

    Some(())
}

pub fn end_instance() {
    crate::log::info!("Stop MITOSIS instance, start cleaning up...");
    unsafe {
        crate::dc_target_service::drop();
        crate::dc_pool_service::drop();
        crate::service_caller_pool::drop();
        crate::service_rpc::drop();
        crate::sp_service::drop();
    };
    end_rdma();
}

/// calculate the session ID of the remote end handler
/// FIXME: this calculation is not very precise, need further refinement
pub fn calculate_session_id(mac_id: usize, thread_id: usize, max_callers: usize) -> usize {
    mac_id * max_callers + thread_id
}

use os_network::ud::UDHyperMeta;

/// Connect the RPC session to the remote nodes
///
/// Note: This function should be called after the start_instance call
///
/// TODO: We should wrap a global lock, since this function can called by many applications
///
pub fn probe_remote_rpc_end(
    remote_machine_id: usize,
    connect_info: crate::rpc_service::HandlerConnectInfo,
) -> core::option::Option<()> {
    let len = unsafe { crate::get_rpc_caller_pool_ref().len() };
    for i in 0..len {
        let session_id = calculate_session_id(remote_machine_id, i, len);
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
            )?;
    }
    Some(())
}
