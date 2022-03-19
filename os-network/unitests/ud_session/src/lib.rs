#![no_std]

extern crate alloc;

use core::fmt::Write;

use KRdmaKit::cm::SidrCM;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use rust_kernel_linux_util as log;

use os_network::block_on;
use os_network::conn::{Conn, Factory, MetaFactory};
use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::timeout::Timeout;

use mitosis_macros::declare_global;

use krdma_test::*;

const DEFAULT_QD_HINT: u64 = 74;

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);

use os_network::datagram::ud_receiver::*;
use os_network::bytes::ToBytes;
use core::pin::Pin;

fn test_ud_session() {
    log::info!("UD session test started!"); 

    let driver = unsafe { KDRIVER::get_ref() };
    let nic = driver.devices().into_iter().next().unwrap();

    // initialization 
    let factory = UDFactory::new(nic).unwrap();    
    let ctx = factory.get_context();

    // server UD
    let server_ud = factory.create(()).unwrap();

    // expose it 
    let service_id: u64 = 0;
    let ctrl = RCtrl::create(service_id, &ctx).unwrap();
    ctrl.reg_ud(DEFAULT_QD_HINT as usize, server_ud.get_qp());

    // the client part 
    let gid = ctx.get_gid_as_string();    
    let (endpoint, key) = factory.create_meta((gid, service_id, DEFAULT_QD_HINT)).unwrap();
    log::info!("check endpoint, key: {:?}, {}", endpoint, key); 
}

#[krdma_test(test_ud_session)]
fn ctx_init() {
    unsafe {
        KDRIVER::init(KDriver::create().unwrap());
    }
}

#[krdma_drop]
fn ctx_drop() {
    unsafe {
        KDRIVER::drop();
    }
}
