#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use core::pin::Pin;

use linux_kernel_module::c_types::*;

use KRdmaKit::KDriver;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::mem::{RMemPhy, Memory};

use mitosis_macros::{declare_module_param, declare_global};

declare_module_param!(nic_count, c_uint);
declare_module_param!(service_id_base, c_uint);
declare_module_param!(memory_size, c_ulong);

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util as log;

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);
declare_global!(RCONTEXTS, alloc::boxed::Box<alloc::vec::Vec<KRdmaKit::device::RContext<'static>>>);
declare_global!(GLOBAL_MEMORY, alloc::boxed::Box<KRdmaKit::mem::RMemPhy>);

struct Module<'a> {
    rctrls: Vec<Pin<Box<RCtrl<'a>>>>,
}

impl linux_kernel_module::KernelModule for Module<'_> {
    
    fn init() -> linux_kernel_module::KernelResult<Self> {
        
        unsafe { KDRIVER::init(KDriver::create().unwrap()) };
        unsafe { RCONTEXTS::init(Box::new(Vec::new())) };

        let driver = unsafe { KDRIVER::get_ref() };
        let mut rctrls = Vec::new();
        
        for i in 0..nic_count::read() {
            let ctx = driver
                .devices()[i as usize]
                .open()
                .unwrap();
            log::info!("the rkey of ctx {}: {}", i, unsafe { ctx.get_rkey() });
            unsafe { 
                RCONTEXTS::get_mut().push(ctx);
            };
        }

        let mut pmem = RMemPhy::new(memory_size::read() as usize);
        let paddr = pmem.get_pa(0);
        unsafe {
            GLOBAL_MEMORY::init(Box::new(pmem));
        }
        log::info!("the physical addr {}", paddr);

        for i in 0..nic_count::read() {
            let service_id = service_id_base::read();
            log::info!("create RCtrl {} on device {}", service_id, i);
            let rctrl = RCtrl::create(service_id as u64, unsafe { &RCONTEXTS::get_ref()[i as usize] });
            rctrls.push(rctrl.unwrap());
        }
        
        Ok(Self {
            rctrls: rctrls,
        })
    }
}

impl Drop for Module<'_> {

    fn drop(&mut self) {
        self.rctrls.clear();
        unsafe {
            RCONTEXTS::drop();
            KDRIVER::drop();
            GLOBAL_MEMORY::drop();
        }
    }
}

linux_kernel_module::kernel_module!(
    Module,
    author: b"xmm",
    description: b"Test test framework in the kernel",
    license: b"GPL"
);

unsafe impl Sync for Module<'_> {}
