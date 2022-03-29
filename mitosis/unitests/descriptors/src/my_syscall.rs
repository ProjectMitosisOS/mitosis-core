use KRdmaKit::rust_kernel_rdma_base::*;
use mitosis::descriptors::{Descriptor, ReadMeta};
use mitosis::resume::fork_prepare_impl;
use mitosis::rpc_handlers::payload::ForkResume;
use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;
use crate::*;
use os_network::msg::UDMsg as RMemory;


use os_network::{block_on};
use os_network::bytes::{ToBytes};
use os_network::msg::UDMsg;
use os_network::serialize::Serialize;
use os_network::ud::UDHyperMeta;

pub struct MySyscallHandler {
    // For RPC handler and page fault handler
    session_id: usize,

    pool_id: usize,
    parent_descriptor: Option<Descriptor>,
    local_mem: RMemory,

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
            session_id: 30,
            pool_id: 0,
            parent_descriptor: None,
            local_mem: RMemory::new(1024 * 1024 * 4, 0),
            file: file as *mut _,
        })
    }

    #[allow(non_snake_case)]
    #[inline]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        match cmd {
            0 => self.test_fork_prepare(arg),                                // fork_prepare
            1 => self.test_fork_resume(arg),                                 // fork_resume
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

const TEST_HANDLER_ID: usize = 73;

impl MySyscallHandler {
    /// Param of syscall
    ///
    /// Param handler_id: Prepare key
    /// Param nic_idx:    Used for assign the RNIC0 or RNIC1
    #[inline(always)]
    fn test_fork_prepare(&self, _arg: c_ulong) -> c_long {
        crate::log::debug!("In test of fork prepare");
        let (handler_id, nic_idx) = (TEST_HANDLER_ID, self.pool_id % 2);
        fork_prepare_impl(handler_id, nic_idx)
    }


    /// Test the (de)serialization of RegDescriptor
    ///
    /// Param gid: Remote IB mac address.
    ///
    /// Param handler_id: Prepare key. Should be the same as `fork_prepare`
    ///
    /// Param remote_nic_idx: Parent nic idx. Should be the same as `fork_prepare`.
    /// And this idx should align with gid
    #[inline(always)]
    fn test_fork_resume(&mut self, _arg: c_ulong) -> c_long {
        let local_pool_id = self.pool_id;
        let nic_num = 2;
        let ctx = unsafe {
            mitosis::get_rpc_caller_pool_ref()
                .get_caller_context(local_pool_id)
                .unwrap()
        };
        let (parent_gid, handler_id, remote_nic_idx) = (
            // os_network::rdma::RawGID::new(String::from("fe80:0000:0000:0000:248a:0703:009c:7c94")).unwrap(),
            os_network::rdma::RawGID::new(ctx.get_gid_as_string()).unwrap(),
            TEST_HANDLER_ID,
            local_pool_id % nic_num); // TODO: refine the mapping relation

        let hyper_meta = UDHyperMeta {
            gid: parent_gid,
            service_id: mitosis::rdma_context::SERVICE_ID_BASE + remote_nic_idx as u64,
            qd_hint: mitosis::rpc_service::QD_HINT_BASE as u64 + remote_nic_idx as u64,
        };
        // 1. setup session
        if unsafe { mitosis::get_rpc_caller_pool_mut() }
            .connect_session_at(local_pool_id,
                                self.session_id, // Notice: it is very important to ensure that session ID is unique!
                                hyper_meta).is_none() {
            log::error!("bad connect!");
            return 0;
        }

        // 2. RPC fetching
        if self.call_fork_resume(handler_id, 0).is_none() {
            log::error!("call fork_resume rpc error!");
            return -1;
        }
        // 3. Apply this descriptor into child process
        unsafe { self.get_parent_meta().apply_to(self.file) };
        return 0;
    }
}

impl MySyscallHandler {
    #[inline]
    fn call_fork_resume(&mut self,
                        handler_id: usize, auth_key: usize) -> Option<()> {
        let ctx = unsafe {
            mitosis::get_rpc_caller_pool_ref()
                .get_caller_context(self.pool_id)
                .unwrap()
        };
        let caller = unsafe {
            mitosis::rpc_caller_pool::CallerPool::get_global_caller(self.pool_id)
                .expect("the caller should be properly inited")
        };
        let payload: ForkResume = ForkResume { handler_id, auth_key };
        caller.sync_call::<ForkResume>(
            self.session_id, // remote session ID
            mitosis::rpc_handlers::RPCId::ForkResume as _, // RPC ID
            payload,  // send an arg of u64
        ).unwrap();
        caller.register_recv_buf(UDMsg::new(4096, 73)).unwrap(); // should succeed
        let remote_meta = match block_on(caller) {
            Ok((_, reply)) => {
                ReadMeta::deserialize(&reply)
            }
            Err(e) => {
                log::error!("client receiver reply err {:?}", e);
                None
            }
        };
        if remote_meta.is_none() {
            return None;
        }
        let remote_meta = remote_meta.unwrap();
        // 2. rmem_cpy to fetch remote descriptor
        let point = caller.get_ss(self.session_id).unwrap().0.get_ss_meta();

        // TODO: extrac dc into global variable
        let dc = unsafe { crate::GLOBAL_DC::get_mut() };
        let pa = self.local_mem.get_pa();
        let lkey = unsafe { ctx.get_lkey() };
        remote_mem::rmem_cpy(dc, pa, remote_meta.addr,
                             remote_meta.length as _, lkey,
                             point);
        // Deserialize the output
        self.parent_descriptor = Some(Descriptor::deserialize(self.local_mem.get_bytes()).unwrap());
        Some(())
    }
}


impl MySyscallHandler {
    #[inline]
    unsafe fn handle_page_fault(&mut self, vmf: *mut mitosis::bindings::vm_fault) -> c_int {
        use mitosis::bindings::{pmem_alloc_page, PMEM_GFP_HIGHUSER};
        // virtual address
        let fault_addr = (*vmf).address;
        let meta = Some(self.get_parent_meta());
        let phy = meta.and_then(|m| {
            m.lookup_pg_table(fault_addr).and_then(|phy_addr| {
                let new_page_p = pmem_alloc_page(PMEM_GFP_HIGHUSER);
                let new_page_pa = mitosis::bindings::pmem_page_to_phy(new_page_p) as u64;
                let ctx = unsafe {
                    mitosis::get_rpc_caller_pool_ref()
                        .get_caller_context(self.pool_id)
                        .unwrap()
                };
                let caller = unsafe {
                    mitosis::rpc_caller_pool::CallerPool::get_global_caller(self.pool_id)
                        .expect("the caller should be properly inited")
                };
                let point = caller.get_ss(self.session_id).unwrap().0.get_ss_meta();
                // let mut remote_mm: RemoteMemManager = RemoteMemManager::create(ctx, point);
                // rmem_cpy to fetch remote page
                let dc = unsafe { crate::GLOBAL_DC::get_mut() };

                remote_mem::rmem_cpy(dc, new_page_pa, phy_addr, 4096, unsafe { ctx.get_lkey() },
                                     point);
                (*vmf).page = new_page_p as *mut _;
                Some(phy_addr)
            })
        });
        // check the results
        match phy {
            Some(_phy) => 0,
            None => self.handle_remote_page_fault(vmf)
        }
    }

    /// Remote page fault handler, forming the fallback path if the target page is not in the PTE (`phy_addr == 0`)
    #[inline(always)]
    unsafe fn handle_remote_page_fault(&mut self, vmf: *mut mitosis::bindings::vm_fault) -> c_int {
        let fault_addr = (*vmf).address;
        log::error!("fallback into remote page fault. fault addr: 0x{:x}", fault_addr);

        //get_backed_file_name
        // let vma = Task::new().get_memory_descriptor().find_vma(fault_addr).unwrap();
        // let vma = VMA::new(vma);
        // let fname = vma.get_backed_file_name();
        // if fname.is_some() {
        //     log::error!("fault file name:{}", fname.unwrap());
        // }
        // TODO: Fetch page from remote (rpage_fault)
        mitosis::bindings::FaultFlags::SIGSEGV.bits() as linux_kernel_module::c_types::c_int
    }

    #[inline]
    unsafe fn get_parent_meta(&self) -> &Descriptor {
        self.parent_descriptor.as_ref().unwrap()
    }
}
