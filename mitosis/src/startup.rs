use crate::rdma_context::*;

#[allow(unused_imports)]
use crate::linux_kernel_module;

use rust_kernel_linux_util::timer::KTimer;

use alloc::vec::Vec;
use alloc::sync::Arc;

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
        crate::log::info!(
            "[check]: Prefetch optimization is enabled, prefetch sz {}.",
            crate::PREFETCH_STEP
        );
    } else {
        crate::log::info!("[check]: Disable prefetching.");
    }

    if cfg!(feature = "page-cache") {
        crate::log::info!("[check]: Cache remote page table optimization is enabled.")
    } else {
        crate::log::info!("[check]: Not cache remote page table.")
    }

    crate::log::info!("********* All configuration check passes !*********");
}

pub fn init_mitosis(config: &crate::Config) -> core::option::Option<()> {
    crate::log::info!("Try to start MITOSIS instance, init global services");
    check_global_configurations();

    unsafe {
        crate::mac_id::init(config.machine_id);
        crate::max_caller_num::init(config.max_core_cnt);
        crate::max_nics_used::init(config.num_nics_used);
        crate::max_cluster_size::init(config.max_cluster_size);
    };


    start_rdma(config).expect("fail to create RDMA context");
    crate::log::info!("Initialize RDMA context done");

    // high-level RDMA-related data structures

    // UD factory
    unsafe {
        use os_network::datagram::ud::*;

        let mut ud_factories = Vec::new();
        for c in crate::rdma_contexts::get_ref() {
            ud_factories.push(Arc::new(UDFactory::new(c)));
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

        crate::access_info_service::init(crate::dc_pool::AccessInfoPool::new(config.max_core_cnt));
    };

    // RC factory
    unsafe {
        use os_network::rdma::rc::*;

        let mut rc_factories = Vec::new();
        for c in crate::rdma_contexts::get_ref() {
            rc_factories.push(RCFactory::new(c));
        }
        crate::rc_factories::init(rc_factories);
    }

    // global lock
    {
        let mut locks = Vec::new();
        for _ in 0..config.max_core_cnt {
            locks.push(crate::linux_kernel_module::mutex::LinuxMutex::new(()));
        }

        for i in 0..locks.len() {
            locks[i].init();
        }

        unsafe { crate::global_locks::init(locks) };
    }


    // DCQP & target pool
    unsafe {
        crate::dc_pool_service::init(
            crate::dc_pool::DCPool::new(config).expect("Failed to create DCQP pool"),
        );

        #[cfg(feature = "prefetch")]
        crate::dc_pool_service_async::init(crate::lock_bundler::LockBundler::new(
            crate::dc_pool::DCPool::new(&config)
                .expect("Failed to create DCQP pool for the async ops"),
        ));

        crate::dc_target_service::init(
            crate::dc_pool::DCTargetPool::new(config).expect("Failed to create DC Target pool"),
        )
    };

    // Global shadow process service
    unsafe { crate::sp_service::init(crate::shadow_process_service::ShadowProcessService::new()) };

    // Memory pool for the shadow process service
    // The context is not important here as we only allocate a slice of memory
    unsafe { crate::mem_pool::init(crate::mem_pools::MemPool::new(config.mem_pool_size, crate::get_rdma_context_ref(0).unwrap().clone())) };

    // cache for storing the remote page table cache
    unsafe {
        crate::global_pt_cache::init(crate::remote_pt_cache::RemotePageTableCache::default())
    };


    unsafe {
        crate::service_rpc::init(Default::default());
        crate::service_caller_pool::init(Default::default())
    };
    // TODO: other services


    // establish an RPC to myself
    // for i in 0..config.rpc_threads_num {
    // probe_remote_rpc_end(
    //     0,
    //     unsafe { crate::service_rpc::get_ref() }
    //         .get_connect_info(0)
    //         .expect("Self RPC handler connection info uninitialized"),
    // )
    // .expect("failed to connect to my RPC handlers!");
    // }
    // crate::log::info!("Probe myself RPC handlers done");


    Some(())
}

pub fn init_rpc(config: &crate::Config,
                rpc_worker: extern "C" fn(*mut crate::linux_kernel_module::c_types::c_void) -> i32) -> core::option::Option<()> {
    // RPC service
    unsafe {
        crate::service_rpc::init(
            crate::rpc_service::Service::new_with_worker(config, rpc_worker).expect("Failed to create the RPC service. "),
        );
    };
    crate::log::info!("RPC service initializes done");

    // RPC caller pool
    unsafe {
        crate::service_caller_pool::init(
            crate::rpc_caller_pool::CallerPool::new(config)
                .expect("Failed to create the RPC caller pool"),
        )
    };

    crate::log::info!("Start waiting for the RPC servers to start...");
    crate::rpc_service::wait_handlers_ready_barrier(config.rpc_threads_num);
    crate::log::info!("All RPC thread handlers initialized!");

    Some(())
}

pub fn start_instance(config: crate::Config) -> core::option::Option<()> {
    let timer = KTimer::new();

    init_mitosis(&config)?;
    init_rpc(&config, crate::rpc_service::Service::worker)?;

    crate::log::info!(
        "All initialization done, takes {} ms",
        timer.get_passed_usec() / 1000
    );
    Some(())
}

pub fn end_instance() {
    crate::log::info!("Stop MITOSIS instance, start cleaning up...");
    unsafe {
        crate::ud_factories::drop();
        crate::dc_factories::drop();

        crate::service_rpc::drop();
        crate::access_info_service::drop();

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

        crate::global_locks::drop();
    };
    end_rdma();

    crate::log::info!("MITOSIS instance stopped, bye~")
}

/// calculate the session ID of the remote end handler
/// FIXME: this calculation is not very precise, need further refinement
pub fn calculate_session_id(mac_id: usize, thread_id: usize, max_callers: usize) -> usize {
    mac_id * max_callers + thread_id
}

use os_network::{ud::UDHyperMeta, KRdmaKit::comm_manager::Explorer};

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
        let my_session_id = calculate_session_id(unsafe { *crate::mac_id::get_ref() }, i, len);
        let gid = Explorer::string_to_gid(&connect_info.gid).ok()?;
        // assert_ne!(session_id, my_session_id);

        unsafe { crate::get_rpc_caller_pool_mut() }.connect_session_at(
            i,
            session_id, // Notice: it is very important to ensure that session ID is unique!
            my_session_id,
            UDHyperMeta {
                // the remote machine's RDMA gid. Since we are unit test, we use the local gid
                gid,
                service_id: connect_info.service_id,
                qd_hint: connect_info.qd_hint,
                local_port: connect_info.local_port,
            },
        )?;
    }
    Some(())
}
