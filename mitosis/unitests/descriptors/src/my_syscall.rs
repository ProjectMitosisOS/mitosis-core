use KRdmaKit::Profile;
use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;
use crate::*;


use os_network::block_on;
use os_network::ud::UDHyperMeta;

pub struct MySyscallHandler;

#[allow(non_upper_case_globals)]
impl FileOperations for MySyscallHandler {
    #[inline]
    fn open(
        _file: *mut crate::linux_kernel_module::bindings::file,
    ) -> crate::linux_kernel_module::KernelResult<Self> {
        Ok(Self)
    }

    #[allow(non_snake_case)]
    #[inline]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        match cmd {
            0 => self.test_fork_prepare(arg),                                // fork_prepare
            1 => self.test_fork_resume(arg),                   // fork_resume
            _ => {
                crate::log::error!("unknown system call command ID {}", cmd);
                -1
            }
        }
    }

    #[inline]
    fn mmap(&mut self, _vma_p: *mut vm_area_struct) -> c_int {
        unimplemented!();
    }
}

impl MySyscallHandler {
    #[inline(always)]
    fn test_fork_prepare(&self, _arg: c_ulong) -> c_long {
        crate::log::debug!("In test of fork prepare\n");
        0
    }


    /// Test the (de)serialization of RegDescriptor
    #[inline(always)]
    fn test_fork_resume(&self, _arg: c_ulong) -> c_long {
        crate::log::debug!("In test of fork resume\n");
        let pool_idx = 0;
        let context = unsafe {
            mitosis::get_rpc_caller_pool_ref()
                .get_caller_context(pool_idx)
                .unwrap()
        };
        let gid = os_network::rdma::RawGID::new(context.get_gid_as_string()).unwrap();
        const SESSION_IDX: usize = 66;
        const RNIC0: u64 = mitosis::rdma_context::SERVICE_ID_BASE;
        // const RNIC1: u64 = mitosis::rdma_context::SERVICE_ID_BASE + 1;
        const T0: u64 = mitosis::rpc_service::QD_HINT_BASE as u64;
        // const T1: u64 = mitosis::rpc_service::QD_HINT_BASE as u64 + 1;
        let hyper_meta = UDHyperMeta {
            gid,
            service_id: RNIC0,
            qd_hint: T0,
        };
        let mut profile = Profile::new();

        let _ = unsafe { mitosis::get_rpc_caller_pool_mut() }
            .connect_session_at(
                pool_idx,
                SESSION_IDX, // Notice: it is very important to ensure that session ID is unique!
                hyper_meta,
            ).expect("failed to connect the endpoint");
        let caller = unsafe {
            mitosis::rpc_caller_pool::CallerPool::get_global_caller(pool_idx)
                .expect("the caller should be properly inited")
        };
        let run_cnt = 16;
        profile.reset_timer();
        for _ in 0..run_cnt {
            caller.sync_call::<u64>(
                SESSION_IDX, // remote session ID
                mitosis::rpc_handlers::RPCId::SwapDescriptor as _, // RPC ID
                0xffffffff as u64,  // send an arg of u64
            ).unwrap();
            let _ = block_on(caller);
        }
        profile.tick_record(0);
        profile.increase_op(run_cnt as _);
        profile.report(1);
        0
    }
}
