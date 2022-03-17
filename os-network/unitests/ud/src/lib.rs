#![no_std]

extern crate alloc;

use core::fmt::Write;

use KRdmaKit::cm::SidrCM;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use rust_kernel_linux_util as log;

use os_network::block_on;
use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::{Datagram, Factory, Receiver};
use os_network::timeout::Timeout;

use mitosis_macros::declare_global;

use krdma_test::*;

const DEFAULT_QD_HINT: u64 = 74;
const MSG_SIZE: usize = 512;

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);

/// A test on `UDFactory`
/// Pre-requisition: `ctx_init`
fn test_ud_factory() {
    let driver = unsafe { KDRIVER::get_ref() };
    let nic = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available");

    let factory = UDFactory::new(nic);
    if factory.is_none() {
        log::error!("fail to create ud factory");
        return;
    }

    let factory = factory.unwrap();
    let ud = factory.create(());
    if ud.is_err() {
        log::error!("fail to create ud qp");
        return;
    }
    log::info!("test ud factory passes");
}

use os_network::datagram::ud_receiver::*;

/// A test on one-sided operation on `UDDatagram`
/// Pre-requisition: `ctx_init`
fn test_ud_two_sided() {
    let timeout_usec = 1000_000;
    let driver = unsafe { KDRIVER::get_ref() };
    let nic = driver.devices().into_iter().next().unwrap();

    let factory = UDFactory::new(nic).unwrap();
    let ctx = factory.get_context();

    let server_ud = factory.create(()).unwrap();
    let mut client_ud = factory.create(()).unwrap();

    let service_id: u64 = 0;
    let ctrl = RCtrl::create(service_id, &ctx).unwrap();
    ctrl.reg_ud(DEFAULT_QD_HINT as usize, server_ud.get_qp());
    let gid = ctx.get_gid_as_string();

    let path_res = factory.get_context().explore_path(gid, service_id).unwrap();
    let mut sidr_cm = SidrCM::new(ctx, core::ptr::null_mut()).unwrap();
    let endpoint = sidr_cm
        .sidr_connect(path_res, service_id, DEFAULT_QD_HINT)
        .unwrap();

    let mut ud_receiver = UDReceiver::new(server_ud);
    for _ in 0..12 {
        match ud_receiver
            .post_recv_buf(UDMsg::new(MSG_SIZE), unsafe { ctx.get_lkey() }) { 
            Ok(_) => {},
            Err(e) => log::error!("post recv buf err: {:?}", e),
        }
    }        

    let mut send_msg = UDMsg::new(MSG_SIZE);
    write!(&mut send_msg, "hello world").unwrap();

    let result = client_ud.post_msg(&endpoint, &send_msg, unsafe { &ctx.get_lkey() });
    if result.is_err() {
        log::error!("fail to post message");
        return;
    }

    // check the message has been sent
    let mut timeout_client_ud = Timeout::new(client_ud, timeout_usec);
    let result = block_on(&mut timeout_client_ud);
    if result.is_err() {
        log::error!("polling send ud qp with error: {:?}", result.err().unwrap());
    }

    // start to receive
    let mut timeout_server_receiver = Timeout::new(ud_receiver, timeout_usec);
    let result = block_on(&mut timeout_server_receiver);
    if result.is_err() {
        log::error!(
            "polling receive ud qp with error: {:?}",
            result.err().unwrap()
        );
    }

    log::info!("finally check the content: {:?}", send_msg.get_bytes()); 
}

#[krdma_test(test_ud_factory, test_ud_two_sided)]
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
