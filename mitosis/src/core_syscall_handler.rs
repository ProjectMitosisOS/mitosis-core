use alloc::string::String;
use core::option::Option;

use crate::descriptors::FastDescriptor;
use crate::linux_kernel_module::c_types::*;
use crate::remote_paging::{AccessInfo, RemotePagingService};
use crate::syscalls::FileOperations;

use os_network::block_on;
use os_network::bytes::ToBytes;
use os_network::timeout::TimeoutWRef;

#[allow(unused_imports)]
use crate::linux_kernel_module;
use crate::rpc_service::HandlerConnectInfo;
use crate::startup::probe_remote_rpc_end;

#[allow(dead_code)]
struct ResumeDataStruct {
    handler_id: usize,
    descriptor: crate::descriptors::Descriptor,
    access_info: crate::remote_paging::AccessInfo,
}

struct CallerData {
    ping_img: bool,
    prepared_key: Option<usize>,
    resume_related: Option<ResumeDataStruct>,
}

impl Default for CallerData {
    fn default() -> Self {
        Self {
            ping_img: false,
            prepared_key: None,
            resume_related: None,
        }
    }
}

/// The MitosisSysCallService has the following two jobs:
///  1. handle up parent/child system calls
///  2. register the corresponding pagefault handler
pub struct MitosisSysCallHandler {
    caller_status: CallerData,
    // structure to encapsulate caller's status
    my_file: *mut crate::bindings::file,
}

impl Drop for MitosisSysCallHandler {
    fn drop(&mut self) {
        if !self.caller_status.ping_img {
            self.caller_status.prepared_key.map(|k| {
                crate::log::info!("unregister prepared process {}", k);
                let process_service = unsafe { crate::get_sps_mut() };
                process_service.unregister(k);
            });
        } else {

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

        // Tricky: Walk can be accelerated here!
        {
            let task = crate::kern_wrappers::task::Task::new();
            task.generate_mm();
        }

        Ok(Self {
            my_file: file as *mut _,
            caller_status: Default::default(),
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
            LibMITOSISCmd::ResumeLocal => self.syscall_local_resume(arg),
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
                    let res = self.syscall_local_resume_w_rpc(mac_id as _, handler_id as _);
                    profile.tick_record(0);
                    profile.increase_op(1);
                    profile.report(1);
                    res
                } else {
                    self.syscall_local_resume_w_rpc(mac_id as _, handler_id as _)
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
                self.syscall_connect_session(machine_id as _, &gid, nic_id as _)
            }
            LibMITOSISCmd::PreparePing => self.syscall_prepare(arg, true),
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

const TIMEOUT_USEC: i64 = 1000_000; // 1s

/// The system call parts
impl MitosisSysCallHandler {
    #[inline]
    fn syscall_prepare(&mut self, key: c_ulong, ping_img: bool) -> c_long {
        if self.caller_status.prepared_key.is_some() {
            crate::log::error!("We don't support multiple fork yet. ");
            return -1;
        }
        self.caller_status.ping_img = ping_img;

        let process_service = unsafe { crate::get_sps_mut() };
        let res = if cfg!(feature = "cow") {
            process_service.add_myself_cow(key as _)
        } else {
            process_service.add_myself_copy(key as _)
        };

        if res.is_some() {
            self.caller_status.prepared_key = Some(key as _);
            crate::log::debug!("prepared buf sz {}KB", res.unwrap() / 1024);
            return 0;
        }
        return -1;
    }

    #[inline]
    fn syscall_local_resume(&mut self, handler_id: c_ulong) -> c_long {
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
                descriptor: descriptor.clone(),
                // access info cannot failed to create
                access_info: AccessInfo::new(&descriptor.machine_info).unwrap(),
            });
            descriptor.apply_to(self.my_file);
            return 0;
        }
        return -1;
    }

    /// This is just a sample test function
    #[inline]
    fn syscall_local_resume_w_rpc(&mut self, machine_id: c_ulong, handler_id: c_ulong) -> c_long {
        if self.caller_status.resume_related.is_some() {
            crate::log::error!("We don't support multiple resume yet. ");
            return -1;
        }
        let cpu_id = crate::get_calling_cpu_id();
        // send an RPC to the remote to query the descriptor address
        let caller = unsafe {
            crate::rpc_caller_pool::CallerPool::get_global_caller(cpu_id)
                .expect("the caller should be properly initialized")
        };

        // ourself must have been connected in the startup process
        let remote_session_id = unsafe {
            crate::startup::calculate_session_id(
                machine_id as _,
                cpu_id,
                *crate::max_caller_num::get_ref(),
            )
        };

        caller
            .sync_call::<usize>(
                remote_session_id,
                crate::rpc_handlers::RPCId::Query as _,
                handler_id as _,
            )
            .unwrap();

        let mut timeout_caller = TimeoutWRef::new(caller, TIMEOUT_USEC);

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

                        // fetch the descriptor with one-sided RDMA
                        let desc_buf = RemotePagingService::remote_descriptor_fetch(
                            d,
                            caller,
                            remote_session_id,
                        );
                        crate::log::debug!("sanity check fetched desc_buf {:?}", desc_buf.is_ok());
                        if desc_buf.is_err() {
                            crate::log::error!("failed to fetch descriptor {:?}", desc_buf.err());
                            return -1;
                        }

                        // deserialize
                        let des = {
                            let des = FastDescriptor::deserialize(desc_buf.unwrap().get_bytes());
                            if des.is_none() {
                                crate::log::error!("failed to deserialize descriptor");
                                return -1;
                            }
                            des.unwrap().to_descriptor()
                        };

                        let access_info = AccessInfo::new(&des.machine_info);
                        if access_info.is_none() {
                            crate::log::error!("failed to create access info");
                            return -1;
                        }

                        des.apply_to(self.my_file);

                        self.caller_status.resume_related = Some(ResumeDataStruct {
                            handler_id: handler_id as _,
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
    }

    #[inline]
    fn syscall_connect_session(
        &mut self,
        machine_id: usize,
        gid: &alloc::string::String,
        nic_idx: usize,
    ) -> c_long {
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
    #[inline(always)]
    unsafe fn handle_page_fault(&mut self, vmf: *mut crate::bindings::vm_fault) -> c_int {
        let fault_addr = (*vmf).address;
        // crate::log::debug!("fault addr 0x{:x}", fault_addr);
        let resume_related = self.caller_status.resume_related.as_ref().unwrap();
        let new_page = resume_related
            .descriptor
            .read_remote_page(fault_addr, &resume_related.access_info);
        match new_page {
            Some(new_page_p) => {
                (*vmf).page = new_page_p as *mut _;
                0
            }
            None => {
                crate::log::error!("[handle_page_fault] Failed to read the remote page");
                crate::bindings::FaultFlags::SIGSEGV.bits() as linux_kernel_module::c_types::c_int
            }
        }
    }
}

unsafe impl Sync for MitosisSysCallHandler {}

unsafe impl Send for MitosisSysCallHandler {}
