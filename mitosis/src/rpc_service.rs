use alloc::boxed::Box;
use alloc::vec::Vec;

use os_network::KRdmaKit::comm_manager::Explorer;
use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::kthread::JoinHandler;

use rust_kernel_linux_util::linux_kernel_module::c_types::{c_int, c_void};

use crate::linux_kernel_module;

use core::sync::atomic::{AtomicUsize, Ordering};

static RPC_HANDLER_READY_NUM: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn wait_handlers_ready_barrier(wait_num: usize) {
    loop {
        if RPC_HANDLER_READY_NUM.load(Ordering::SeqCst) >= wait_num {
            return;
        }
    }
}

#[derive(Debug)]
pub struct HandlerConnectInfo {
    pub gid: alloc::string::String,
    pub service_id: u64,
    pub qd_hint: usize,
    pub local_port: u8,
}

impl HandlerConnectInfo {
    /// Create a remote connection information, so as to connect to remote
    /// # Arguments
    /// * gid: the remote GID
    /// * nic_idx: the remote NIC ID used
    /// * tid: the remote RPC id
    #[inline]
    pub fn create(gid: &alloc::string::String, nic_idx: u64, tid: usize) -> Self {
        Self::create_with_port(gid, nic_idx, tid, 1)
    }

    #[inline]
    pub fn create_with_port(gid: &alloc::string::String, nic_idx: u64, tid: usize, port_num: u8) -> Self {
        let (service_id, qd_hint) = (
            crate::rdma_context::SERVICE_ID_BASE as u64
                // FIXME! we assume that the remote machine has the same number of NICs used
                // as ourself, so we can directly calculate here. 
                // Otherwise, we need to pass the service_id here, 
                // but it will expose too much to the upper-layer applications
                + nic_idx % unsafe { (*crate::max_nics_used::get_ref()) as u64 },
            Service::calculate_qd_hint(tid) as _,
        );
        Self {
            gid: gid.clone(),
            service_id,
            qd_hint,
            local_port: port_num,
        }
    }
}

impl Clone for HandlerConnectInfo {
    fn clone(&self) -> HandlerConnectInfo {
        Self {
            gid: self.gid.clone(),
            service_id: self.service_id,
            qd_hint: self.qd_hint,
            local_port: self.local_port,
        }
    }
}

/// RPC service implements a UDRpcHook defined in os-network
/// Note: It must be created after all the RDMA context has been initialized.
/// Session ID mapping:
///    The RPC caller that connects to the machine (mac_id)'s (thread_id)'s:
///    * session_id = mac_id * max_rpc_threads + thread_id
pub struct Service {
    threads: Vec<JoinHandler>,
    connect_infos: Vec<HandlerConnectInfo>,
}

impl core::fmt::Debug for Service {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MitosisRPCService")
            .field("threads_num", &self.threads.len())
            .field("connect_infos", &self.connect_infos)
            .finish()
    }
}

impl Service {
    pub fn get_connect_info(&self, idx: usize) -> core::option::Option<HandlerConnectInfo> {
        self.connect_infos.get(idx).map(|c| c.clone())
    }

    pub fn calculate_qd_hint(idx: usize) -> usize {
        QD_HINT_BASE + idx
    }
}

impl Service {
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        let mut res = Self {
            threads: Vec::new(),
            connect_infos: Vec::new(),
        };

        // init the RPC connect info
        for i in 0..config.rpc_threads_num {
            let nic_to_use = i % config.num_nics_used;
            let local_context = unsafe { crate::rdma_contexts::get_ref().get(nic_to_use).unwrap() };
            let qd_hint = Self::calculate_qd_hint(i);

            res.connect_infos.push(HandlerConnectInfo {
                gid: Explorer::gid_to_string(&local_context.query_gid(config.default_nic_port, 0).ok()?),
                service_id: crate::rdma_context::SERVICE_ID_BASE + nic_to_use as u64,
                qd_hint: qd_hint,
                local_port: config.default_nic_port,
            });
        }

        for i in 0..config.rpc_threads_num {
            let arg = Box::new(ThreadCTX {
                id: i,
                nic_to_use: i % config.num_nics_used,
            });
            let arg_ptr = Box::into_raw(arg);

            let builder = kthread::Builder::new()
                .set_name(alloc::format!("MITOSIS RPC handler Thread {}", i))
                .set_parameter(arg_ptr as *mut c_void);
            let handler = builder.spawn(Self::worker).ok()?;
            res.threads.push(handler);
        }

        crate::log::debug!("RPC service creation done: {:?}", res);

        Some(res)
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        let total = self.threads.len();
        for _ in 0..total {
            self.threads.pop().map(|handler| handler.join());
        }
    }
}

struct ThreadCTX {
    pub(crate) id: usize,
    pub(crate) nic_to_use: usize,
}

use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::future::*;
use os_network::Factory;

use rust_kernel_linux_util::timer::KTimer;

pub const QD_HINT_BASE: usize = 12;

use super::rpc_handlers::*;

impl Service {
    const YIELD_THRESHOLD: usize = 10000;
    const YIELD_TIME_USEC: i64 = 2000; // 1ms

    #[allow(non_snake_case)]
    extern "C" fn worker(ctx: *mut c_void) -> c_int {
        let arg = unsafe { Box::from_raw(ctx as *mut ThreadCTX) };

        // init the UD RPC hook
        type UDRPCHook<'a> =
            os_network::rpc::hook::RPCHook<'a, UDDatagram, UDReceiver, UDFactory>;

        let local_context = unsafe { crate::rdma_contexts::get_ref().get(arg.nic_to_use).unwrap() };
        let lkey = unsafe { local_context.lkey() };

        crate::log::info!(
            "MITOSIS RPC thread {} started, listing on gid: {:?}",
            arg.id,
            local_context.query_gid(1, 0).unwrap(), // WTX: use the default port to query gid
        );

        let server_ud = unsafe {
            crate::ud_factories::get_ref()
                .get(arg.nic_to_use)
                .expect("failed to query the factory")
                .create(UDCreationMeta { port: 1 }) // WTX: use the default port to create server-side ud
                .expect("failed to create server UD")
        };
        let temp_ud = server_ud.clone();

        unsafe {
            crate::ud_service::get_mut()
                .get(arg.nic_to_use)
                .unwrap()
                .reg_qp(Service::calculate_qd_hint(arg.id), &server_ud.get_qp())
        };

        let mut rpc_server = UDRPCHook::new(
            unsafe { crate::get_ud_factory(arg.nic_to_use).unwrap().clone() },
            server_ud.clone(),
            UDReceiverFactory::new()
                .set_qd_hint(Service::calculate_qd_hint(arg.id) as _)
                .create(temp_ud),
        );

        // register the callbacks
        rpc_server
            .get_mut_service()
            .register(RPCId::Nil as _, handle_nil);
        rpc_server
            .get_mut_service()
            .register(RPCId::Echo as _, handle_echo);
        rpc_server
            .get_mut_service()
            .register(RPCId::Query as _, handle_descriptor_addr_lookup);

        // register msg buffers
        // pre-most receive buffers
        for _ in 0..2048 {
            // 64 is the header
            match rpc_server.post_msg_buf(UDMsg::new(4096, 0, server_ud.get_qp().ctx().clone())) {
                Ok(_) => {}
                Err(e) => crate::log::error!("post recv buf err: {:?}", e),
            }
        }
        RPC_HANDLER_READY_NUM.fetch_add(1, core::sync::atomic::Ordering::SeqCst);

        let mut counter = 0;
        let mut timer = KTimer::new();

        while !kthread::should_stop() {
            match rpc_server.poll() {
                Ok(Async::Ready(_)) => {}
                Ok(_NotReady) => {}
                Err(e) => crate::log::error!(
                    "RPC handler {} meets an error {:?}, status {:?}",
                    arg.id,
                    e,
                    rpc_server
                ),
            };
            counter += 1;
            if core::intrinsics::unlikely(counter > Self::YIELD_THRESHOLD) {
                if core::intrinsics::unlikely(timer.get_passed_usec() > Self::YIELD_TIME_USEC) {
                    kthread::yield_now();
                    timer.reset();
                }
                counter = 0;
            }
        }

        crate::log::info!(
            "MITOSIS RPC thread {} ended. rpc status: {:?} ",
            arg.id,
            rpc_server
        );
        0
    }
}
