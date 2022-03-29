use alloc::sync::Arc;
use core::pin::Pin;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;
use KRdmaKit::cm::EndPoint;
use KRdmaKit::device::RContext;
use KRdmaKit::mem::{Memory, RMemPhy};
use KRdmaKit::rust_kernel_rdma_base::*;
use mitosis::descriptors::{Descriptor, DescriptorFactoryService, RDMADescriptor, ReadMeta};
use mitosis::get_descriptor_pool_mut;
use mitosis::kern_wrappers::mm::{PhyAddrType, VirtAddrType};
use mitosis::kern_wrappers::task::Task;
use mitosis::kern_wrappers::vma::VMA;
use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;
use crate::*;
use os_network::rdma::payload;


use os_network::{block_on, Conn, Factory, rdma};
use os_network::msg::UDMsg;
use os_network::rdma::dc::{DCConn, DCFactory};
use os_network::rdma::RawGID;
use os_network::serialize::Serialize;
use os_network::timeout::Timeout;
use os_network::ud::UDHyperMeta;

pub struct MySyscallHandler {
    // storing parent meta info. Fetch by calling `get_parent_meta`
    meta_buf: Option<RMemPhy>,
    file: *mut mitosis::bindings::file,
}

unsafe impl Sync for MySyscallHandler {}

unsafe impl Send for MySyscallHandler {}

static mut MY_VM_OP: mitosis::bindings::vm_operations_struct = unsafe {
    core::mem::transmute([0u8; core::mem::size_of::<mitosis::bindings::vm_operations_struct>()])
};

#[allow(dead_code)]
unsafe extern "C" fn open_handler(_area: *mut mitosis::bindings::vm_area_struct) {}

#[allow(dead_code)]
unsafe extern "C" fn page_fault_handler(vmf: *mut mitosis::bindings::vm_fault) -> c_int {
    let handler: *mut MySyscallHandler = (*(*vmf).vma).vm_private_data as *mut _;
    (*handler).handle_page_fault(vmf)
}

#[allow(non_upper_case_globals)]
impl FileOperations for MySyscallHandler {
    #[inline]
    fn open(
        file: *mut crate::linux_kernel_module::bindings::file,
    ) -> crate::linux_kernel_module::KernelResult<Self> {
        unsafe {
            MY_VM_OP = Default::default();
            MY_VM_OP.open = Some(open_handler);
            MY_VM_OP.fault = Some(page_fault_handler);
            MY_VM_OP.access = None;
        };

        Ok(Self {
            meta_buf: Some(RMemPhy::new(1024 * 1024)),
            file: file as *mut _,
        })
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
    fn mmap(&mut self, vma_p: *mut vm_area_struct) -> c_int {
        unsafe {
            (*vma_p).vm_private_data = (self as *mut Self).cast::<c_void>();
            (*vma_p).vm_ops = &mut MY_VM_OP as *mut mitosis::bindings::vm_operations_struct as *mut _;
        }
        0
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
    fn test_fork_resume(&mut self, _arg: c_ulong) -> c_long {
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
        // 1. Two-sided RDMA to fetch the address and length information
        caller.sync_call::<u64>(
            SESSION_IDX, // remote session ID
            mitosis::rpc_handlers::RPCId::ForkResume as _, // RPC ID
            0xffffffff as u64,  // send an arg of u64
        ).unwrap();
        caller.register_recv_buf(UDMsg::new(4096, 73)).unwrap(); // should succeed
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

        // 2. rmem_cpy to fetch remote descriptor
        let point = caller.get_ss(SESSION_IDX).unwrap().0.get_ss_meta();
        let mut remote_mem = RemoteMemManager::create(ctx, point);
        let local = self.meta_buf.as_mut().unwrap();
        remote_mem.rmem_cpy(local.get_pa(0), dst.addr, dst.length as _, 5000);
        compiler_fence(SeqCst);


        // 3. Apply this descriptor into child process
        DescriptorFactoryService::resume_from_descriptor(self.file, unsafe { self.get_parent_meta() });
        return 0;
    }
}

/// Simple remote memory, implemented by DC primitive
struct RemoteMemManager<'a> {
    dc_factory: DCFactory<'a>,
    remote_point: &'a EndPoint,
}

impl<'a> RemoteMemManager<'a> {
    pub fn create(ctx: &'a RContext<'a>, point: &'a EndPoint) -> Self {
        Self {
            dc_factory: DCFactory::new(ctx),
            remote_point: point,
        }
    }

    /// Remote mem_cpy, implemented by one-sided RDMA read.
    ///
    /// Since we are in kernel, both of dst and src address should be physical
    #[inline]
    pub fn rmem_cpy(&mut self, dst: PhyAddrType, src: PhyAddrType, len: u64, timeout_usec: i64) -> isize {
        type DCReqPayload = payload::Payload<ib_dc_wr>;
        let point = &self.remote_point;
        let lkey = unsafe { self.dc_factory.get_context().get_lkey() };
        // TODO: DC create should not in the critical path!
        let mut dc = self.dc_factory.create(()).unwrap();
        let mut payload = DCReqPayload::default()
            .set_laddr(dst)
            .set_raddr(src)// copy from src into dst
            .set_sz(len as _)
            .set_lkey(lkey)
            .set_rkey(point.mr.get_rkey())
            .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
            .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
            .set_ah(point);
        let mut payload = unsafe { Pin::new_unchecked(&mut payload) };
        os_network::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());
        if dc.post(&payload.as_ref()).is_err() {
            log::error!("unable to post read qp");
            return -1;
        }
        let mut timeout = Timeout::new(dc,
                                       timeout_usec as _);
        if block_on(&mut timeout).is_err() {
            log::error!("polling dc qp with error");
            return -1;
        }
        return 0;
    }
}


impl MySyscallHandler {
    #[inline]
    unsafe fn handle_page_fault(&mut self, vmf: *mut mitosis::bindings::vm_fault) -> c_int {
        use mitosis::bindings::{pmem_phys_to_virt, pmem_page_to_phy, pmem_alloc_page, PMEM_GFP_HIGHUSER};
        // virtual address
        let fault_addr = (*vmf).address;
        let meta = Some(self.get_parent_meta());
        let phy = meta.as_ref().and_then(|m| {
            m.lookup_pg_table(fault_addr).and_then(|phy_addr| {
                // One-sided RDMA Read
                let new_page_p = pmem_alloc_page(PMEM_GFP_HIGHUSER);
                let new_page_pa = mitosis::bindings::pmem_page_to_phy(new_page_p) as u64;

                // TODO: omit `pa==0` case while calling `fork_prepare()`
                if phy_addr > 0 {
                    let ctx = unsafe {
                        mitosis::get_rpc_caller_pool_ref()
                            .get_caller_context(0)
                            .unwrap()
                    };
                    const SESSION_IDX: usize = 66;
                    let caller = unsafe {
                        mitosis::rpc_caller_pool::CallerPool::get_global_caller(0)
                            .expect("the caller should be properly inited")
                    };
                    let point = caller.get_ss(SESSION_IDX).unwrap().0.get_ss_meta();
                    let mut remote_mm: RemoteMemManager = RemoteMemManager::create(ctx, point);
                    // rmem_cpy to fetch remote page
                    remote_mm.rmem_cpy(new_page_pa, phy_addr, 4096, 5000);
                    (*vmf).page = new_page_p as *mut _;
                    Some(phy_addr)
                } else {
                    // FIXME: How to handle the case `pa==0` ?
                    let mm = Task::new().get_memory_descriptor();
                    // let vma = mm.find_vma(fault_addr).unwrap();
                    let vma = VMA::new(mm.find_vma(fault_addr).unwrap());
                    log::debug!("Find bad, fault addr: 0x{:x}, is stack: {}, vma start: 0x{:x}, size :{}",
                        fault_addr, vma.is_stack(), vma.get_start(), vma.get_sz());
                    None
                }
            })
        });
        // check the results
        match phy {
            Some(phy) => {
                // log::debug!(
                //     "Check fault address 0x{:x} => to dumped physical address: 0x{:x}",
                //     fault_addr, phy
                // );
                0
            }
            None => {
                log::error!("Failed to find the fault address: 0x{:x}", fault_addr);
                // If it is the stack or the heap, we should manually handles the fault like the original page_fault_handler
                // TO be implemented
                mitosis::bindings::FaultFlags::SIGSEGV.bits() as linux_kernel_module::c_types::c_int
            }
        }
    }

    #[inline]
    unsafe fn get_parent_meta(&self) -> Arc<Descriptor> {
        let local = self.meta_buf.as_ref().unwrap();
        Arc::<Descriptor>::from_raw(local.get_ptr() as _)
    }
}
