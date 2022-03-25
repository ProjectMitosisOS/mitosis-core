use alloc::boxed::Box;
use alloc::vec::Vec;

use rust_kernel_linux_util::kthread;
use rust_kernel_linux_util::kthread::JoinHandler;

use rust_kernel_linux_util::linux_kernel_module::c_types::{c_int, c_void};

use crate::linux_kernel_module;

/// RPC service implements a UDRpcHook defined in os-network
pub struct Service {
    threads: Vec<JoinHandler>,
}

impl Service {
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        let mut res = Self {
            threads: Vec::new(),
        };

        for i in 0..config.rpc_threads_num {
            let arg = Box::new(ThreadCTX { id: i });
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
}

impl Service {
    const YIELD_THRESHOLD: usize = 10000;
    const YIELD_TIME_USEC: i64 = 2000; // 1ms

    extern "C" fn worker(ctx: *mut c_void) -> c_int {
        let arg = unsafe { Box::from_raw(ctx as *mut ThreadCTX) };
        crate::log::debug!("MITOSIS RPC thread {} started. ", arg.id);

        while !kthread::should_stop() {
            kthread::yield_now();
        }

        crate::log::debug!("MITOSIS RPC thread {} ended. ", arg.id);
        0
    }
}
