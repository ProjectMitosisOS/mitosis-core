use alloc::boxed::Box;
use alloc::vec::Vec;

use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::kthread::JoinHandler;

use rust_kernel_linux_util::linux_kernel_module::c_types::{c_int, c_void};

use crate::linux_kernel_module;

/// RPC service implements a UDRpcHook defined in os-network
/// Note: It must be created after all the RDMA context has been initialized.
pub struct Service {
    threads: Vec<JoinHandler>,
}

impl Service {
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        let mut res = Self {
            threads: Vec::new(),
        };

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
        crate::log::debug!("MITOSIS RPC thread {} started. ", arg.id);

        // init the UD RPC hook
        type UDRPCHook<'a, 'b> =
            os_network::rpc::hook::RPCHook<'a, 'b, UDDatagram<'a>, UDReceiver<'a>, UDFactory<'a>>;

        let local_context = unsafe { crate::rdma_contexts::get_ref().get(arg.nic_to_use).unwrap() };
        let lkey = unsafe { local_context.get_lkey() };

        let server_ud = unsafe {
            crate::ud_factories::get_ref()
                .get(arg.nic_to_use)
                .expect("failed to query the factory")
                .create(())
                .expect("failed to create server UD")
        };
        let temp_ud = server_ud.clone();

        unsafe {
            crate::rdma_cm_service::get_mut()
                .get(arg.nic_to_use)
                .unwrap()
                .reg_ud(QD_HINT_BASE + arg.id, server_ud.get_qp())
        };

        let mut rpc_server = UDRPCHook::new(
            unsafe { crate::ud_factories::get_ref().get(arg.nic_to_use).unwrap() },
            server_ud,
            UDReceiverFactory::new()
                .set_qd_hint((QD_HINT_BASE + arg.id) as _)
                .set_lkey(lkey)
                .create(temp_ud),
        );

        // register the callbacks
        rpc_server
            .get_mut_service()
            .register(RPCId::Nil as _, handle_nil);
        rpc_server
            .get_mut_service()
            .register(RPCId::Echo as _, handle_echo);

        // register msg buffers
        // pre-most receive buffers
        for _ in 0..1024 {
            // 64 is the header
            match rpc_server.post_msg_buf(UDMsg::new(4096, 73)) {
                Ok(_) => {}
                Err(e) => crate::log::error!("post recv buf err: {:?}", e),
            }
        }

        let mut counter = 0;
        let mut timer = KTimer::new();

        while !kthread::should_stop() {
            match rpc_server.poll() {
                Ok(Async::Ready(_)) => {}
                Ok(_NotReady) => {}
                Err(e) => crate::log::error!("RPC handler {} meets an error {:?}", arg.id, e),
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

        crate::log::debug!(
            "MITOSIS RPC thread {} ended. rpc status: {:?} ",
            arg.id,
            rpc_server
        );
        0
    }
}
