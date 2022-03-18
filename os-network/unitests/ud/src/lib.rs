#![no_std]

extern crate alloc;

use core::fmt::Write;

use KRdmaKit::cm::SidrCM;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use rust_kernel_linux_util as log;

use os_network::block_on;
use os_network::conn::{Conn, Factory};
use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
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
use os_network::bytes::ToBytes;
use core::pin::Pin;

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
        // 64 is the header
        match ud_receiver.post_recv_buf(UDMsg::new(MSG_SIZE + 64, 73), unsafe { ctx.get_lkey() }) {
            Ok(_) => {}
            Err(e) => log::error!("post recv buf err: {:?}", e),
        }
    }

    let mut send_msg = UDMsg::new(MSG_SIZE, 73);
    write!(&mut send_msg, "hello world").unwrap();

    let mut send_req = send_msg
        .to_ud_wr(&endpoint)
        .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
        .set_lkey(unsafe { ctx.get_lkey() });

    // pin the request to set the sge_ptr properly. 
    let mut send_req = unsafe { Pin::new_unchecked(&mut send_req) }; 
    os_network::rdma::payload::Payload::<ib_ud_wr>::finalize(send_req.as_mut()); 

    let result = client_ud.post(&send_req.as_ref());

    if result.is_err() {
        log::error!("fail to post message");
        return;
    }

    // check the message has been sent
    let mut timeout_client_ud = Timeout::new(client_ud, timeout_usec);
    let result = block_on(&mut timeout_client_ud);
    if result.is_err() {
        log::error!("polling send ud qp with error: {:?}", result.err().unwrap());
    } else {
        log::info!("post msg done");
    }

    // start to receive
    let mut timeout_server_receiver = Timeout::new(ud_receiver, timeout_usec);
    let result = block_on(&mut timeout_server_receiver);
    if result.is_err() {
        log::error!(
            "polling receive ud qp with error: {:?}",
            result.err().unwrap()
        );
    } else {
        let received_msg = unsafe { result.unwrap().get_bytes().truncate(0).unwrap() };
        log::info!(
            "Get received msg: {:?}",
            // if the result is correct, then the truncate size should be 40
            received_msg
        );
        let received_msg = unsafe { received_msg.truncate(40).unwrap() };
        log::debug!("received msg: {} {:?} ", received_msg.len(), received_msg);

        assert!(
            unsafe { received_msg.clone_and_resize(send_msg.get_bytes().len()) }.unwrap()
                == unsafe { send_msg.get_bytes().clone() }
        );
    }

    log::info!(
        "finally check the content:{} {:?}",
        send_msg.len(),
        send_msg.get_bytes()
    );
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
