#![no_std]

extern crate alloc;

use rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use mitosis::core_syscall_handler::*;
use mitosis::startup::{end_instance, start_instance};
use mitosis::syscalls::*;

#[allow(dead_code)]
struct Module {
    service: SysCallsService<MitosisSysCallHandler>,
}

use os_network::block_on;

impl linux_kernel_module::KernelModule for Module {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        let mut config: mitosis::Config = Default::default();
        config.set_num_nics_used(2).set_rpc_threads(2);
        
        assert!(start_instance(config.clone()).is_some());

        // now make some simple checks for self-called RPC
        // now we can call!
        let pool_idx = 0;
        let caller = unsafe {
            mitosis::rpc_caller_pool::CallerPool::get_global_caller(pool_idx)
                .expect("the caller should be properly inited")
        };

        let self_session_id = mitosis::startup::calculate_session_id(
            config.get_machine_id(),
            pool_idx,
            config.get_max_core_cnt(),
        );

        caller
            .sync_call(
                self_session_id,                                      // remote session ID
                mitosis::rpc_handlers::RPCId::Echo as _, // RPC ID
                0xffffffff as u64,                       // send an arg of u64
            )
            .unwrap();

        let res = block_on(caller);
        match res {
            Ok(v) => {
                let (msg, reply) = v; // msg, reply
                log::debug!("sanity check rpc two call result: {:?}", reply);
                caller.register_recv_buf(msg).expect("register msg buffer cannot fail");
            }
            Err(e) => log::error!("client call error: {:?}", e),
        };

        Ok(Self {
            service: SysCallsService::<MitosisSysCallHandler>::new()?,
        })
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        end_instance();
        log::info!("drop System call modules")
    }
}

linux_kernel_module::kernel_module!(
    Module,
    author: b"xmm",
    description: b"A kernel module for testing the core fork functions!",
    license: b"GPL"
);
