#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use krdma_test::{krdma_test, krdma_drop};

use KRdmaKit::KDriver;
use KRdmaKit::ctrl::RCtrl;

use mitosis_macros::{declare_module_param, declare_global};

declare_module_param!(nic_count, u64);
declare_module_param!(service_id_base, u64);

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util as log;

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);
declare_global!(RCTRLS, alloc::boxed::Box<alloc::vec::Vec<core::pin::Pin<alloc::boxed::Box<KRdmaKit::ctrl::RCtrl<'static>>>>>);
declare_global!(RCONTEXTS, alloc::boxed::Box<alloc::vec::Vec<KRdmaKit::device::RContext<'static>>>);

fn module_main() {
    let driver = unsafe { KDRIVER::get_ref() };

    for i in 0..nic_count::read() {
        log::info!("create context {} on nic {}", i, i);
        let ctx = driver
            .devices()[i as usize]
            .open()
            .unwrap();
        unsafe { RCONTEXTS::get_mut().push(ctx) };
    }

    for i in 0..nic_count::read() {
        let service_id = service_id_base::read();
        log::info!("create RCtrl {} on context {}", service_id, i);
        let rctrl = RCtrl::create(service_id, unsafe { &RCONTEXTS::get_ref()[i as usize] }).unwrap();
        unsafe { RCTRLS::get_mut().push(rctrl) };
    }
}

#[krdma_test(module_main)]
fn ctx_init() {
    unsafe {
        KDRIVER::init(KDriver::create().unwrap());
        RCONTEXTS::init(Box::new(Vec::new()));
        RCTRLS::init(Box::new(Vec::new()));
    }
}

#[krdma_drop]
fn module_drop() {
    unsafe {
        RCTRLS::drop();
        RCONTEXTS::drop();
        KDRIVER::drop();
    };
    log::info!("context drop");
}
