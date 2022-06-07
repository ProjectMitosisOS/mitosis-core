use crate::rdma_context::*;

#[allow(unused_imports)]
use crate::linux_kernel_module;

use rust_kernel_linux_util::timer::KTimer;

use alloc::vec::Vec;

pub fn check_global_configurations() {
    if cfg!(feature = "eager-resume") {
        crate::log::info!("[check]: eager resume mode is on.")
    } else {
        crate::log::info!("[check]: use on-demand resume mode.")
    }

    if cfg!(feature = "cow") {
        crate::log::info!("[check]: Parent is using copy-on-write (COW) mode.")
    } else {
        crate::log::info!("[check]: Parent is using copy to dump the image.")
    }

    if cfg!(feature = "prefetch") {
        crate::log::info!("[check]: Prefetch optimization is enabled.")
    } else {
        crate::log::info!("[check]: Disable prefetching.")
    }

    if cfg!(feature = "page-cache") {
        crate::log::info!("[check]: Cache remote page table optimization is enabled.")
    } else {
        crate::log::info!("[check]: Not cache remote page table.")
    }

    crate::log::info!("********* All configuration check passes !*********");
}

pub fn start_instance(config: crate::Config) -> core::option::Option<()> {
    crate::log::info!("Try to start MITOSIS instance, init global services");
    check_global_configurations();

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

        #[cfg(feature = "prefetch")]
        crate::dc_pool_service_async::init(crate::lock_bundler::LockBundler::new(
            crate::dc_pool::DCPool::new(&config)
                .expect("Failed to create DCQP pool for the async ops"),
        ));

        crate::dc_target_service::init(
            crate::dc_pool::DCTargetPool::new(&config).expect("Failed to create DC Target pool"),
        )
    };

    // Global shadow process service
    unsafe { crate::sp_service::init(crate::shadow_process_service::ShadowProcessService::new()) };

    //  Memory pool for the shadow process service
    unsafe { crate::mem_pool::init(crate::mem_pools::MemPool::new(config.mem_pool_size)) };

    // cache for storing the remote page table cache
    unsafe { crate::global_pt_cache::init(crate::remote_pt_cache::RemotePageTableCache::default()) };

    // TODO: other services

    crate::log::info!("Start waiting for the RPC servers to start...");
    crate::rpc_service::wait_handlers_ready_barrier(config.rpc_threads_num);
    crate::log::info!("All RPC thread handlers initialized!");

    // establish an RPC to myself
    // for i in 0..config.rpc_threads_num {
    //     probe_remote_rpc_end(
    //         1 + config.max_cluster_size * i,
    //         unsafe { crate::service_rpc::get_ref() }
    //             .get_connect_info(i)
    //             .expect("Self RPC handler connection info uninitialized"),
    //     )
    //     .expect("failed to connect to my RPC handlers!");
    // }
    // crate::log::info!("Probe myself RPC handlers done");

    crate::log::info!(
        "All initialization done, takes {} ms",
        timer.get_passed_usec() / 1000
    );

    Some(())
}

pub fn end_instance() {
    crate::log::info!("Stop MITOSIS instance, start cleaning up...");
    unsafe {
        crate::service_rpc::drop();

        crate::log::debug!("drop dc targets");
        crate::dc_target_service::drop();

        crate::log::debug!("drop dc pool");
        crate::dc_pool_service::drop();

        #[cfg(feature = "prefetch")]
        crate::dc_pool_service_async::drop();

        crate::service_caller_pool::drop();

        crate::log::debug!("drop shadow process service");
        crate::sp_service::drop();
        crate::mem_pool::drop();

        crate::global_pt_cache::drop();
    };
    end_rdma();

    crate::log::info!("MITOSIS instance stopped, byte~")
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
        unsafe { crate::get_rpc_caller_pool_mut() }.connect_session_at(
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
