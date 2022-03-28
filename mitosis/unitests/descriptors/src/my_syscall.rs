use alloc::string::ToString;
use core::pin::Pin;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;
use KRdmaKit::cm::SidrCM;
use KRdmaKit::mem::{Memory, pa_to_va, RMemPhy};
use KRdmaKit::Profile;
use KRdmaKit::rust_kernel_rdma_base::*;
use mitosis::descriptors::{Descriptor, RDMADescriptor, ReadMeta};
use mitosis::get_descriptor_pool_mut;
use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;
use crate::*;
use os_network::rdma::payload;


use os_network::{block_on, Conn, Factory, rdma};
use os_network::bytes::BytesMut;
use os_network::msg::UDMsg;
use os_network::rdma::RawGID;
use os_network::serialize::Serialize;
use os_network::timeout::Timeout;
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
        crate::log::debug!("In test of fork prepare");
        let pool_idx = 0;       // todo: self defined
        let context = unsafe {
            mitosis::get_rpc_caller_pool_ref()
                .get_caller_context(pool_idx)
                .unwrap()
        };
        let des_pool = unsafe { get_descriptor_pool_mut() };
        let raw_gid = RawGID::new(context.get_gid_as_string());
        if raw_gid.is_some() {
            des_pool.put_current_descriptor(73, RDMADescriptor {
                gid: RawGID::new(context.get_gid_as_string()).unwrap(),
                service_id: mitosis::rdma_context::SERVICE_ID_BASE,
                rkey: 64,
            });
            crate::log::debug!("prepare descriptor success", );
        }
        0
    }


    /// Test the (de)serialization of RegDescriptor
    #[inline(always)]
    fn test_fork_resume(&self, _arg: c_ulong) -> c_long {
        crate::log::debug!("In test of fork resume\n");
        let pool_idx = 0;
        let ctx = unsafe {
            mitosis::get_rpc_caller_pool_ref()
                .get_caller_context(pool_idx)
                .unwrap()
        };
        let gid = os_network::rdma::RawGID::new(ctx.get_gid_as_string()).unwrap();
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
        caller.register_recv_buf(UDMsg::new(4096, 73)).unwrap(); // should succeed

        // 1. Two-sided RDMA to fetch the address and length information
        caller.sync_call::<u64>(
            SESSION_IDX, // remote session ID
            mitosis::rpc_handlers::RPCId::SwapDescriptor as _, // RPC ID
            0xffffffff as u64,  // send an arg of u64
        ).unwrap();
        let dst = match block_on(caller) {
            Ok((_, reply)) => {
                ReadMeta::deserialize(&reply)
            }
            Err(e) => {
                log::error!("client receiver reply err {:?}", e);
                None
            }
        };
        if dst.is_none() {
            return -1;
        }
        let dst = dst.as_ref().unwrap();
        log::debug!("addr:0x{:x}, len:{}", dst.addr, dst.length);
        // 2. One sided RDMA read to fetch remote descriptor
        let rkey = unsafe { ctx.get_rkey() };
        let lkey = unsafe { ctx.get_lkey() };
        let client_factory = rdma::dc::DCFactory::new(&ctx);
        let mut dc = client_factory.create(()).unwrap();
        let path_res = client_factory
            .get_context()
            .explore_path(ctx.get_gid_as_string(), RNIC0)
            .unwrap();
        let mut sidr_cm = SidrCM::new(ctx, core::ptr::null_mut()).unwrap();
        let endpoint = sidr_cm
            .sidr_connect(path_res, RNIC0, T0)
            .unwrap();
        const MEM_SZ: usize = 1024 * 1024;
        type DCReqPayload = payload::Payload<ib_dc_wr>;
        let mut local = RMemPhy::new(MEM_SZ);
        let mut payload = DCReqPayload::default()
            .set_laddr(local.get_pa(0))
            .set_raddr(dst.addr)
            .set_sz(dst.length as _)
            .set_lkey(lkey)
            .set_rkey(rkey)
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
            .set_ah(&endpoint);
        let timeout_usec = 5000;
        let mut payload = unsafe { Pin::new_unchecked(&mut payload) };
        os_network::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());
        let res = dc.post(&payload.as_ref());
        if res.is_err() {
            log::error!("unable to post read qp");
            return -1;
        }

        let mut timeout = Timeout::new(dc, timeout_usec);
        let result = block_on(&mut timeout);
        if result.is_err() {
            log::error!("polling dc qp with error");
            return -1;
        }
        compiler_fence(SeqCst);
        let content = unsafe { &*(local.get_ptr() as *mut Descriptor) };

        // 3. Apply this descriptor into child process
        return 0;
    }
}
