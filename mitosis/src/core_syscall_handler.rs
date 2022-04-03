use core::option::Option;

use crate::descriptors::Descriptor;
use crate::linux_kernel_module::c_types::*;
use crate::remote_paging::{AccessInfo, RemotePagingService};
use crate::syscalls::FileOperations;

use os_network::block_on;
use os_network::bytes::ToBytes;
use os_network::timeout::TimeoutWRef;

#[allow(unused_imports)]
use crate::linux_kernel_module;

#[allow(dead_code)]
struct ResumeDataStruct {
    key: usize,
    descriptor: crate::descriptors::Descriptor,
    access_info: crate::remote_paging::AccessInfo,
}

#[derive(Default)]
struct CallerData {
    prepared_key: Option<usize>,
    resume_related: Option<ResumeDataStruct>,
}

/// The MitosisSysCallService has the following two jobs:
///  1. handle up parent/child system calls
///  2. register the corresponding pagefault handler
pub struct MitosisSysCallHandler {
    caller_status: CallerData, // structure to encapsulate caller's status
    my_file: *mut crate::bindings::file,
}

impl Drop for MitosisSysCallHandler {
    fn drop(&mut self) {
        self.caller_status.prepared_key.map(|k| {
            crate::log::debug!("unregister prepared process {}", k);
            let process_service = unsafe { crate::get_sps_mut() };
            process_service.unregister(k);
        });
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

        Ok(Self {
            my_file: file as *mut _,
            caller_status: Default::default(),
        })
    }

    #[allow(non_snake_case)]
    #[inline]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        match cmd {
            mitosis_protocol::CALL_PREPARE => self.syscall_prepare(arg),
            mitosis_protocol::CALL_RESUME_LOCAL => self.syscall_local_resume(arg),
            mitosis_protocol::CALL_RESUME_LOCAL_W_RPC => self.syscall_local_resume_w_rpc(arg),
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
    fn syscall_prepare(&mut self, key: c_ulong) -> c_long {
        if self.caller_status.prepared_key.is_some() {
            crate::log::error!("We don't support multiple fork yet. ");
            return -1;
        }

        let process_service = unsafe { crate::get_sps_mut() };
        let res = process_service.add_myself_copy(key as _);

        if res.is_some() {
            self.caller_status.prepared_key = Some(key as _);
            return 0;
        }
        return -1;
    }

    #[inline]
    fn syscall_local_resume(&mut self, key: c_ulong) -> c_long {
        if self.caller_status.resume_related.is_some() {
            crate::log::error!("We don't support multiple resume yet. ");
            return -1;
        }

        let process_service = unsafe { crate::get_sps_mut() };
        let descriptor = process_service.query_descriptor(key as _);

        if descriptor.is_some() {
            self.caller_status.resume_related = Some(ResumeDataStruct {
                key: key as _,
                descriptor: descriptor.unwrap().clone(),
                // access info cannot failed to create
                access_info: AccessInfo::new(&descriptor.unwrap().machine_info).unwrap(),
            });
            descriptor.unwrap().apply_to(self.my_file);
            return 0;
        }
        return -1;
    }

    /// This is just a sample test function
    #[inline]
    fn syscall_local_resume_w_rpc(&mut self, key: c_ulong) -> c_long {
        if self.caller_status.resume_related.is_some() {
            crate::log::error!("We don't support multiple resume yet. ");
            return -1;
        }

        // send an RPC to the remote to query the descriptor address
        let caller = unsafe {
            crate::rpc_caller_pool::CallerPool::get_global_caller(crate::get_calling_cpu_id())
                .expect("the caller should be properly initialized")
        };

        // ourself must have been connected in the startup process
        let remote_session_id = unsafe {
            crate::startup::calculate_session_id(
                *crate::mac_id::get_ref(),
                crate::get_calling_cpu_id(),
                *crate::max_caller_num::get_ref(),
            )
        };

        let key: usize = key as _;

        caller
            .sync_call::<usize>(
                remote_session_id,
                crate::rpc_handlers::RPCId::Query as _,
                key,
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
                        let des = Descriptor::deserialize(desc_buf.unwrap().get_bytes());
                        if des.is_none() {
                            crate::log::error!("failed to deserialize descriptor");
                            return -1;
                        }
                        let des = des.unwrap();
                        crate::log::debug!("sanity check: {:?}", des.machine_info);

                        let access_info = AccessInfo::new(&des.machine_info);
                        if access_info.is_none() { 
                            crate::log::error!("failed to create access info");
                            return -1;
                        }

                        des.apply_to(self.my_file);

                        self.caller_status.resume_related = Some(ResumeDataStruct {
                            key: key as _,
                            descriptor: des,
                            // access info cannot failed to create
                            access_info: access_info.unwrap(),
                        });
                        return 0;
                    }
                    None => {
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

        let remote_addr = self
            .caller_status
            .resume_related
            .as_ref()
            .unwrap()
            .descriptor
            .lookup_pg_table(fault_addr);

        if remote_addr.is_none() {
            // TODO: fallback?
            crate::log::error!("failed to lookup the mapped address 0x{:x}", fault_addr);
            return crate::bindings::FaultFlags::SIGSEGV.bits()
                as linux_kernel_module::c_types::c_int;
        }

        // crate::log::debug!("lookup address {:?}", remote_addr);

        // mapped, do the remote reads:
        use crate::bindings::{pmem_alloc_page, PMEM_GFP_HIGHUSER};

        // TODO; check whether the allocation is good?
        let new_page_p = pmem_alloc_page(PMEM_GFP_HIGHUSER);
        let new_page_pa = crate::bindings::pmem_page_to_phy(new_page_p) as u64;

        let res = crate::remote_paging::RemotePagingService::remote_read(
            new_page_pa,
            remote_addr.unwrap(),
            4096,
            &self
                .caller_status
                .resume_related
                .as_ref()
                .unwrap()
                .access_info,
        );

        match res {
            Ok(_) => {
                (*vmf).page = new_page_p as *mut _;
                0
            }
            Err(e) => {
                crate::log::error!("Failed to read the remote page {:?}", e);
                crate::bindings::FaultFlags::SIGSEGV.bits() as linux_kernel_module::c_types::c_int
            }
        }
    }
}

unsafe impl Sync for MitosisSysCallHandler {}

unsafe impl Send for MitosisSysCallHandler {}
