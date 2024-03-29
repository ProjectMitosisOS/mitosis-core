use alloc::string::String;

use core::option::Option;
#[allow(unused_imports)]
use rust_kernel_linux_util::kthread;

#[allow(unused_imports)]
use crate::descriptors::{ChildDescriptor, ParentDescriptor};

use crate::linux_kernel_module::c_types::*;
use crate::remote_paging::{AccessInfo, RemotePagingService};
use crate::syscalls::FileOperations;

use os_network::bytes::ToBytes;
use os_network::timeout::TimeoutWRef;
use os_network::{block_on, Factory};
use os_network::rdma::rc::RCConn;

#[allow(unused_imports)]
use crate::linux_kernel_module;
use crate::rpc_service::HandlerConnectInfo;
#[cfg(feature = "use_rc")]
use crate::rc_conn_pool::RCConnectInfo;
use crate::startup::probe_remote_rpc_end;

const TIMEOUT_USEC: i64 = 1000_000; // 1s

#[allow(dead_code)]
struct ResumeDataStruct {
    handler_id: usize,
    remote_mac_id: usize,
    descriptor: crate::descriptors::ChildDescriptor,
    access_info: crate::remote_paging::AccessInfo,
}

impl ResumeDataStruct {
    /// Count the number of entries in my page table
    #[allow(dead_code)]
    pub fn pg_table_entry_cnt(&self) -> usize {
        self.descriptor.page_table.len()
    }
}

struct CallerData {
    ping_img: bool,
    prepared_key: Option<usize>,
    fault_page_cnt: usize,
    resume_related: Option<ResumeDataStruct>,
}

impl Default for CallerData {
    fn default() -> Self {
        Self {
            ping_img: false,
            prepared_key: None,
            fault_page_cnt: 0,
            resume_related: None,
        }
    }
}

use crate::rdma_context::SERVICE_ID_BASE;
use core::sync::atomic::AtomicUsize;

/// The MitosisSysCallService has the following two jobs:
///  1. handle up parent/child system calls
///  2. register the corresponding pagefault handler
#[allow(dead_code)]
pub struct MitosisSysCallHandler {
    caller_status: CallerData,
    // structure to encapsulate caller's status
    my_file: *mut crate::bindings::file,

    resume_counter: AtomicUsize,
}

impl Drop for MitosisSysCallHandler {
    fn drop(&mut self) {
        #[cfg(feature = "resume-profile")]
        {
            let pg_fault_sz = self.fault_page_size() / 1024;
            let meta_workingset_sz = self.meta_workingset_size() / 1024;
            let fetch_page_sz = self.fetched_page_size() / 1024;
            crate::log::info!(
                "workingset size {} KB, page fault size {} KB, fetch page size {} KB",
                meta_workingset_sz,
                pg_fault_sz,
                fetch_page_sz
            );
        }
        self.cache_my_pt();

        #[cfg(feature = "prefetch")]
        if self.caller_status.resume_related.is_some() {
            let res = self
                .caller_status
                .resume_related
                .as_mut()
                .unwrap()
                .descriptor
                .prefetcher
                .drain_connections();
            if res.is_ok() {
                unsafe {
                    crate::get_dc_pool_async_service_ref().lock(|p| p.push_one_qp(res.unwrap()))
                };
            }
        }

        self.caller_status.prepared_key.map(|k| {
            if !self.caller_status.ping_img {
                crate::log::info!("unregister prepared process {}", k);
                let process_service = unsafe { crate::get_sps_mut() };
                process_service.unregister(k);
                crate::log::info!("unregister prepared process {} done", k);
            }
        });
        #[cfg(feature = "eager-resume")]
        {
            if let Some(des) = self.caller_status.resume_related.as_ref() {
                let des = &des.descriptor;
                for k in des.eager_fetched_pages.iter() {
                    unsafe { crate::bindings::pmem_free_page(*k as *mut crate::bindings::page) };
                }
            }
        }
    }
}

#[allow(non_upper_case_globals)]
impl FileOperations for MitosisSysCallHandler {
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

        // Tricky: walk can be accelerated here!
        {
            let task = crate::kern_wrappers::task::Task::new();
            task.generate_mm();
        }

        Ok(Self {
            my_file: file as *mut _,
            caller_status: Default::default(),
            resume_counter: AtomicUsize::new(0),
        }) 
    }

    #[allow(non_snake_case)]
    #[inline]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        use crate::bindings::{connect_req_t, resume_remote_req_t, LibMITOSISCmd};
        use linux_kernel_module::bindings::_copy_from_user;
        match cmd {
            LibMITOSISCmd::Nil => 0, // a nill core do nothing
            LibMITOSISCmd::Prepare => self.syscall_prepare(arg, false),
            LibMITOSISCmd::ResumeLocal => unimplemented!(),
            LibMITOSISCmd::ResumeRemote => {
                let mut req: resume_remote_req_t = Default::default();
                unsafe {
                    _copy_from_user(
                        (&mut req as *mut resume_remote_req_t).cast::<c_void>(),
                        arg as *mut c_void,
                        core::mem::size_of_val(&req) as u64,
                    )
                };
                let (mac_id, handler_id) = (req.machine_id, req.handler_id);
                if cfg!(feature = "resume-profile") {
                    let mut profile = crate::KRdmaKit::Profile::new();
                    let res = self.syscall_resume_w_rpc(mac_id as _, handler_id as _);
                    profile.tick_record(0);
                    profile.increase_op(1);
                    profile.report(1);
                    res
                } else {
                    self.syscall_resume_w_rpc(mac_id as _, handler_id as _)
                }
            }
            LibMITOSISCmd::Connect => {
                let mut req: connect_req_t = Default::default();
                unsafe {
                    _copy_from_user(
                        (&mut req as *mut connect_req_t).cast::<c_void>(),
                        arg as *mut c_void,
                        core::mem::size_of_val(&req) as u64,
                    )
                };

                let mut addr_buf: [u8; 39] = [0; 39];
                let addr = {
                    unsafe {
                        _copy_from_user(
                            addr_buf.as_mut_ptr().cast::<c_void>(),
                            req.gid as *mut c_void,
                            39,
                        )
                    };
                    // now get addr of GID format
                    core::str::from_utf8(&addr_buf).unwrap()
                };
                let (machine_id, gid, nic_id) = (req.machine_id, String::from(addr), req.nic_id);

                #[cfg(not(feature = "use_rc"))]
                {
                    self.syscall_connect_session(machine_id as _, &gid, nic_id as _)
                }
                
                #[cfg(feature = "use_rc")]
                {
                    self.syscall_connect_rc(machine_id as _, &gid, nic_id as _) | self.syscall_connect_session(machine_id as _, &gid, nic_id as _)
                }
            }
            LibMITOSISCmd::PreparePing => self.syscall_prepare(arg, true),
            LibMITOSISCmd::NilRPC => {
                let mut req: resume_remote_req_t = Default::default();
                unsafe {
                    _copy_from_user(
                        (&mut req as *mut resume_remote_req_t).cast::<c_void>(),
                        arg as *mut c_void,
                        core::mem::size_of_val(&req) as u64,
                    )
                };
                let (mac_id, handler_id) = (req.machine_id, req.handler_id);
                self.syscall_nil_rpc(mac_id as _, handler_id as _)
            }
            _ => {
                crate::log::error!("unknown system call command ID {}", cmd);
                -1
            }
        }
    }

    #[inline]
    fn mmap(
        &mut self,
        vma_p: *mut rust_kernel_linux_util::linux_kernel_module::bindings::vm_area_struct,
    ) -> c_int {
        unsafe {
            (*vma_p).vm_private_data = (self as *mut Self).cast::<c_void>();
            (*vma_p).vm_ops = &mut MY_VM_OP as *mut crate::bindings::vm_operations_struct as *mut _;
        }
        0
    }
}

/// The system call parts
impl MitosisSysCallHandler {
    #[inline]
    fn syscall_prepare(&mut self, key: c_ulong, ping_img: bool) -> c_long {
        if self.caller_status.prepared_key.is_some() {
            crate::log::error!("This version doesn't support multiple fork yet. ");
            return -1;
        }
        self.caller_status.ping_img = ping_img;

        let process_service = unsafe { crate::get_sps_mut() };
        let res = if cfg!(feature = "cow") {
            process_service.add_myself_cow(key as _)
        } else {
            process_service.add_myself_copy(key as _)
        };

        if res.is_none() {
            return -1;
        }

        // double remote fork on parent is not supported yet
        // so we mark a flag to prevent future re-prepare
        self.caller_status.prepared_key = Some(key as _);
        crate::log::debug!("prepared buf sz {}KB", res.unwrap() / 1024);

        // code for sanity checks
        /*
        let mm = crate::kern_wrappers::task::Task::new().get_memory_descriptor();
        let vma = mm.find_vma(0x5555561692b8);
        if vma.is_none() {
            crate::log::info!("sanity check failed to lookup vma");
        }
        vma.map(|vma| {
            crate::log::info!("sanity check vma {:?}", vma);
        }); */

        /*
        use crate::bindings::VMFlags;
        let mm = Task::new().get_memory_descriptor();
        for mut vma in mm.get_vma_iter() {
            if vma.is_is_anonymous() {
                // reset the flags
                let mut vm_flag = vma.get_flags();
                vm_flag.insert(VMFlags::RESERVE);
                vma.set_raw_flags(vm_flag.bits());

                crate::log::info!(
                    "VMA {:x}-{:x} {:?}, is_annoy {}",
                    vma.get_start(),
                    vma.get_end(),
                    vma.get_flags(),
                    vma.is_is_anonymous()
                );
            }
        } */

        return 0;
    }

    /// Deperacted
    /// This function is only used for testing
    /// will be removed in the future
    #[inline]
    #[allow(dead_code)]
    fn syscall_local_resume(&mut self, _handler_id: c_ulong) -> c_long {
        unimplemented!();
        /*
        if self.caller_status.resume_related.is_some() {
            crate::log::error!("We don't support multiple resume yet. ");
            return -1;
        }

        let process_service = unsafe { crate::get_sps_mut() };
        let descriptor = process_service.query_descriptor(handler_id as _);

        if descriptor.is_some() {
            let descriptor = descriptor.unwrap().to_descriptor();
            self.caller_status.resume_related = Some(ResumeDataStruct {
                handler_id: handler_id as _,
                remote_mac_id: 0,
                descriptor: descriptor.clone(),
                // access info cannot failed to create
                access_info: AccessInfo::new(&descriptor.machine_info).unwrap(),
            });
            descriptor.apply_to(self.my_file);
            return 0;
        }
        return -1; */
    }

    /// This is just a sample test function
    #[inline]
    fn syscall_resume_w_rpc(&mut self, machine_id: c_ulong, handler_id: c_ulong) -> c_long {
        if self.caller_status.resume_related.is_some() {
            crate::log::error!("We don't support multiple resume yet. ");
            return -1;
        }

        //        self.resume_counter
        //            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);

        // let cpu_id = 0;
        let cpu_id = crate::get_calling_cpu_id();
        assert!(cpu_id < unsafe { *(crate::max_caller_num::get_ref()) });

        // ourself must have been connected in the startup process
        let remote_session_id = unsafe {
            crate::startup::calculate_session_id(
                machine_id as _,
                cpu_id,
                *crate::max_caller_num::get_ref(),
            )
        };

        let my_session_id = unsafe {
            crate::startup::calculate_session_id(
                *crate::mac_id::get_ref(),
                cpu_id,
                *crate::max_caller_num::get_ref(),
            )
        };

        // send an RPC to the remote to query the descriptor address
        let caller = unsafe {
            crate::rpc_caller_pool::CallerPool::get_global_caller(cpu_id)
                .expect("the caller should be properly initialized")
        };
        caller.lock(|caller| {
            let res = caller.sync_call::<usize>(
                remote_session_id,
                my_session_id,
                crate::rpc_handlers::RPCId::Query as _,
                handler_id as _,
            );
    
            if res.is_err() {
                crate::log::error!("failed to call {:?}", res);
                crate::log::info!(
                    "sanity check pending reqs {:?}",
                    caller.get_pending_reqs(remote_session_id)
                );
                return -1;
            };
    
            let mut timeout_caller = TimeoutWRef::new(caller, 10 * TIMEOUT_USEC);
    
            use crate::rpc_handlers::DescriptorLookupReply;
            use os_network::serialize::Serialize;
    
            let _reply = match block_on(&mut timeout_caller) {
                Ok((msg, reply)) => {
                    // first re-purpose the data
                    caller
                        .register_recv_buf(msg)
                        .expect("register msg buffer cannot fail");
                    match DescriptorLookupReply::deserialize(&reply) {
                        Some(d) => {
                            crate::log::debug!("sanity check query descriptor result {:?}", d);
    
                            if !d.ready {
                                crate::log::error!("failed to lookup handler id: {:?}", handler_id);
                                return -1;
                            }
                            #[cfg(feature = "resume-profile")]
                            crate::log::info!("meta descriptor size:{} KB", d.sz / 1024);
    
                            // fetch the descriptor with one-sided RDMA
                            let desc_buf = RemotePagingService::remote_descriptor_fetch(
                                d,
                                caller,
                                machine_id,
                            );

                            crate::log::debug!("sanity check fetched desc_buf {:?}", desc_buf.is_ok());
                            if desc_buf.is_err() {
                                crate::log::error!("failed to fetch descriptor {:?}", desc_buf.err());
                                return -1;
                            }
    
                            // deserialize
                            let des = {
                                // optimized version
                                ChildDescriptor::deserialize(desc_buf.unwrap().get_bytes())
                            };
    
                            if des.is_none() {
                                // crate::log::error!("failed to deserialize the child descriptor");
                                return -1;
                            }
    
                            let mut des = des.unwrap();
    
                            let access_info = AccessInfo::new(&des.machine_info);
                            //let access_info =
                            //AccessInfo::new_from_cache(des.machine_info.mac_id, &des.machine_info);
                            if access_info.is_none() {
                                crate::log::error!("failed to create access info");
                                return -1;
                            }
    
                            des.apply_to(self.my_file);
    
                            #[cfg(feature = "page-cache")]
                            // Read the cache from kernel cache
                            if let Some(cached_pg_table) = unsafe {
                                crate::get_pt_cache_ref().lookup(machine_id as _, handler_id as _)
                            } {
                                crate::log::debug!(
                                    "Find one cached page cache with mac id: {}, handler id: {}",
                                    machine_id,
                                    handler_id
                                );
                                des.page_table = cached_pg_table.copy();
                            }
    
                            self.caller_status.resume_related = Some(ResumeDataStruct {
                                handler_id: handler_id as _,
                                remote_mac_id: machine_id as _,
                                descriptor: des,
                                // access info cannot failed to create
                                access_info: access_info.unwrap(),
                            });
                            return 0;
                        }
                        None => {
                            crate::log::error!("Deserialize error");
                            return -1;
                        }
                    }
                }
                Err(e) => {
                    crate::log::error!("client receiver reply err {:?}", e);
                    return -1;
                }
            };
        })
    }

    #[inline]
    fn syscall_connect_session(
        &mut self,
        machine_id: usize,
        gid: &alloc::string::String,
        nic_idx: usize,
    ) -> c_long {
        crate::log::debug!("connect remote machine id: {}", machine_id);
        let info = HandlerConnectInfo::create(gid, nic_idx as _, nic_idx as _);
        match probe_remote_rpc_end(machine_id, info) {
            Some(_) => {
                crate::log::debug!("connect to nic {}@{} success", nic_idx, gid);
                0
            }
            _ => {
                crate::log::error!("failed to connect {}@{} success", nic_idx, gid);
                -1
            }
        }
    }

    #[cfg(feature = "use_rc")]
    #[inline]
    fn syscall_connect_rc(
        &mut self,
        machine_id: usize,
        gid: &alloc::string::String,
        nic_idx: usize,
    ) -> c_long {
        let info = RCConnectInfo::create(gid, nic_idx as _ );
        let len = unsafe { *crate::max_caller_num::get_ref() };
        for i in 0..len {
            let rc_pool = unsafe { crate::get_rc_conn_pool_mut(i).expect("failed get rc conn pool") };
            match rc_pool.create_rc_connection(i, machine_id, info.clone()) {
                Some(_) => {
                    crate::log::debug!("create rc connection success");
                }
                _ => {
                    crate::log::debug!("failed create rc connection");
                    return -1
                }
            }
        }
        0
    }

    #[inline]
    fn syscall_nil_rpc(&mut self, machine_id: c_ulong, handler_id: c_ulong) -> c_long {
        let cpu_id = crate::get_calling_cpu_id();
        assert!(cpu_id < unsafe { *(crate::max_caller_num::get_ref()) });

        // ourself must have been connected in the startup process
        let remote_session_id = unsafe {
            crate::startup::calculate_session_id(
                machine_id as _,
                cpu_id,
                *crate::max_caller_num::get_ref(),
            )
        };

        let my_session_id = unsafe {
            crate::startup::calculate_session_id(
                *crate::mac_id::get_ref(),
                cpu_id,
                *crate::max_caller_num::get_ref(),
            )
        };

        // send an RPC to the remote to query the descriptor address
        let caller = unsafe {
            crate::rpc_caller_pool::CallerPool::get_global_caller(cpu_id)
                .expect("the caller should be properly initialized")
        };
        caller.lock(|caller| {
            let res = caller.sync_call::<usize>(
                remote_session_id,
                my_session_id,
                crate::rpc_handlers::RPCId::Nil as _,
                handler_id as _,
            );
            if res.is_err() {
                crate::log::error!("failed to call {:?}", res);
                crate::log::info!(
                    "sanity check pending reqs {:?}",
                    caller.get_pending_reqs(remote_session_id)
                );
                return -1;
            };
    
            let mut timeout_caller = TimeoutWRef::new(caller, 10 * TIMEOUT_USEC);
    
            use os_network::serialize::Serialize;
            let _reply = match block_on(&mut timeout_caller) {
                Ok((msg, _reply)) => {
                    // first re-purpose the data
                    caller
                        .register_recv_buf(msg)
                        .expect("register msg buffer cannot fail");
                    return 0;
                }
                Err(e) => {
                    crate::log::error!("client receiver reply err {:?}", e);
                    return -1;
                }
            };
        })
    }
}

/// The fault handler parts
static mut MY_VM_OP: crate::bindings::vm_operations_struct = unsafe {
    core::mem::transmute([0u8; core::mem::size_of::<crate::bindings::vm_operations_struct>()])
};

#[allow(dead_code)]
unsafe extern "C" fn open_handler(_area: *mut crate::bindings::vm_area_struct) {}

#[allow(dead_code)]
unsafe extern "C" fn page_fault_handler(vmf: *mut crate::bindings::vm_fault) -> c_int {
    let handler: *mut MitosisSysCallHandler = (*(*vmf).vma).vm_private_data as *mut _;
    (*handler).handle_page_fault(vmf)
}

impl MitosisSysCallHandler {
    /// Core logic of handling the page faults
    #[inline(always)]
    unsafe fn handle_page_fault(&mut self, vmf: *mut crate::bindings::vm_fault) -> c_int {
        let fault_addr = (*vmf).address;
        #[cfg(feature = "resume-profile")]
        self.incr_fault_page_cnt();

        let resume_related = self.caller_status.resume_related.as_mut().unwrap();
        // #[cfg(feature = "page-cache")]
        // let resume_related = self.caller_status.resume_related.as_ref().unwrap();

        #[cfg(feature = "page-cache")]
        let mut miss_page_cache = false;
        let phy_addr = resume_related.descriptor.lookup_pg_table(fault_addr);

        let new_page = {
            if phy_addr.is_none() {
                None
            } else {
                #[cfg(feature = "page-cache")]
                {
                    use crate::remote_mapping::PhysAddr;
                    let phy_addr = phy_addr.unwrap();
                    let phys_addr = PhysAddr::new(phy_addr);
                    // if cache hit
                    if phys_addr.is_cache() {
                        let mut page = crate::kern_wrappers::Page::new_from_raw(
                            phys_addr.convert_to_page() as *mut crate::bindings::page,
                        );

                        if phys_addr.is_ro() {
                            // Read only, mark it as COW directly
                            page.increase_ref_count();
                            Some(page.get_inner())
                        } else {
                            // the page access is read/write
                            // Not read only, then copy into a new page
                            let new_page_p = crate::bindings::pmem_alloc_page(
                                crate::bindings::PMEM_GFP_HIGHUSER,
                            );

                            crate::kern_wrappers::copy_page_content_4k(
                                new_page_p,
                                page.get_inner(),
                            );
                            Some(new_page_p)
                        }
                    } else {
                        // Cache miss, fallback into RDMA read
                        miss_page_cache = true;
                        resume_related
                            .descriptor
                            .read_remote_page(fault_addr, 
                                &resume_related.access_info,
                            )
                    }
                }
                #[cfg(not(feature = "page-cache"))]
                {
                    resume_related
                        .descriptor
                        .read_remote_page(fault_addr, 
                            &resume_related.access_info,
                        )
                }
            }
        };
        match new_page {
            Some(new_page_p) => {
                (*vmf).page = new_page_p as *mut _;
                // update cache
                #[cfg(feature = "page-cache")]
                if miss_page_cache && phy_addr.is_some() {
                    use crate::remote_mapping::{PhysAddr, PhysAddrBitFlag};

                    // let phy_addr = phy_addr.unwrap();
                    // Caching up this page. Just mark as CoW.
                    // We leave the Cache bit setting process to function `caching_pg_table`
                    crate::kern_wrappers::Page::new_from_raw(new_page_p).increase_ref_count();

                    let kernel_va = PhysAddr::encode(
                        new_page_p as crate::kern_wrappers::mm::VirtAddrType,
                        PhysAddrBitFlag::Cache as _,
                    );
                    resume_related
                        .descriptor
                        .page_table
                        .force_map(x86_64::VirtAddr::new(fault_addr), PhysAddr::new(kernel_va));
                }
                0
            }
            None => {
                // check whether the page is anonymous
                let vma = crate::kern_wrappers::vma::VMA::new(&mut *((*vmf).vma));
                for &vd in &resume_related.descriptor.vma {
                    if vd.is_anonymous && (vma.get_start() == vd.get_start()) {
                        let new_page_p =
                            crate::bindings::pmem_alloc_page(crate::bindings::PMEM_GFP_HIGHUSER);

                        (*vmf).page = new_page_p as *mut _;
                        return 0;
                    }
                }

                crate::log::debug!(
                    "[handle_page_fault] Failed to read the remote page, fault addr: 0x{:x}",
                    fault_addr
                );
                crate::bindings::FaultFlags::SIGSEGV.bits() as linux_kernel_module::c_types::c_int
            }
        }
    }

    #[allow(dead_code)]
    #[inline]
    fn incr_fault_page_cnt(&mut self) {
        self.caller_status.fault_page_cnt += 1;
    }

    /// Page fault size (in Bytes)
    #[allow(dead_code)]
    #[inline]
    fn fault_page_size(&self) -> usize {
        self.caller_status.fault_page_cnt * 4096 as usize
    }

    #[allow(dead_code)]
    #[inline]
    fn meta_workingset_size(&self) -> usize {
        if let Some(meta) = self.caller_status.resume_related.as_ref() {
            meta.pg_table_entry_cnt() * 4096 as usize
        } else {
            0
        }
    }

    #[cfg(feature = "resume-profile")]
    fn fetched_page_size(&self) -> usize {
        if let Some(meta) = self.caller_status.resume_related.as_ref() {
            meta.descriptor.remote_fetched_page_count * 4096 as usize
        } else {
            0
        }
    }

    /// Cache my page table in the kernel
    /// Called only when the process exit
    #[inline]
    fn cache_my_pt(&self) {
        #[cfg(feature = "page-cache")]
        if let Some(resume_related) = self.caller_status.resume_related.as_ref() {
            // copy to the kernel cache

            let pg_table = resume_related.descriptor.page_table.copy();
            unsafe {
                crate::get_pt_cache_mut().insert(
                    resume_related.remote_mac_id,
                    resume_related.handler_id,
                    pg_table,
                );
            }
        }
    }
}

unsafe impl Sync for MitosisSysCallHandler {}

unsafe impl Send for MitosisSysCallHandler {}
