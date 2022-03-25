#![no_std]

extern crate alloc;

use core::fmt::Write;

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use rust_kernel_linux_util as log;

use os_network::block_on;
use os_network::conn::{Factory, MetaFactory};
use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::timeout::Timeout;

use mitosis_macros::declare_global;

use krdma_test::*;

const DEFAULT_QD_HINT: u64 = 73;
const MAX_SEND_MSG: usize = 64;
const MAX_RECV_MSG: usize = 1024; // receive msg should be no smaller than MAX_SEND_MSG + 40

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);

use os_network::bytes::ToBytes;
use os_network::datagram::ud_receiver::*;
use os_network::rpc::*;

fn test_ud_session() {
    log::info!("UD session test started!");
    let timeout_usec = 1000_000;

    let driver = unsafe { KDRIVER::get_ref() };
    let ctx = driver.devices().into_iter().next().unwrap().open().unwrap();

    // initialization
    let factory = UDFactory::new(&ctx);
    let ctx = factory.get_context();

    // server UD
    let server_ud = factory.create(()).unwrap();
    let client_ud = factory.create(()).unwrap();

    // expose it
    let service_id: u64 = 0;
    let ctrl = RCtrl::create(service_id, &ctx).unwrap();
    ctrl.reg_ud(DEFAULT_QD_HINT as usize, server_ud.get_qp());

    // init the receiver
    let mut ud_receiver = UDReceiverFactory::new()
        .set_qd_hint(0)
        .set_lkey(unsafe { ctx.get_lkey() })
        .create(server_ud);

    for _ in 0..12 {
        // 64 is the header
        match ud_receiver.post_recv_buf(UDMsg::new(MAX_RECV_MSG, 73)) {
            Ok(_) => {}
            Err(e) => log::error!("post recv buf err: {:?}", e),
        }
    }

    // the client part
    let gid = os_network::rdma::RawGID::new(ctx.get_gid_as_string()).unwrap();

    let (endpoint, key) = factory
        .create_meta(UDHyperMeta {
            gid: gid,
            service_id: service_id,
            qd_hint: DEFAULT_QD_HINT,
        })
        .unwrap();
    log::info!("check endpoint, key: {:?}, {}", endpoint, key);

    // send a hello world to the session
    let mut client_session = client_ud.create((endpoint, key)).unwrap();
    let mut request = UDMsg::new(MAX_SEND_MSG, 73);
    write!(&mut request, "hello world").unwrap();

    let result = client_session.post(&request, 64, true);
    if result.is_err() {
        log::error!("fail to post message");
        return;
    }
    // check the message has been sent
    let mut timeout_client = Timeout::new(client_session, timeout_usec);
    let result = block_on(&mut timeout_client);
    if result.is_err() {
        log::error!("polling send ud qp with error: {:?}", result.err().unwrap());
    } else {
        log::info!("post msg done");
    }

    // now receive the request
    let mut timeout_server_receiver = Timeout::new(ud_receiver, timeout_usec);
    let result = block_on(&mut timeout_server_receiver);
    if result.is_err() {
        log::error!(
            "polling receive ud qp with error: {:?}",
            result.err().unwrap()
        );
    } else {
        let received_msg = unsafe { result.unwrap().get_bytes().truncate_header(0).unwrap() };
        log::info!(
            "Get received msg: {:?}",
            // if the result is correct, then the truncate size should be 40
            received_msg
        );
        let received_msg = unsafe { received_msg.truncate_header(40).unwrap() };
        log::debug!("received msg: {} {:?} ", received_msg.len(), received_msg);

        assert!(
            unsafe { received_msg.clone_and_resize(request.get_bytes().len()) }.unwrap()
                == unsafe { request.get_bytes().clone() }
        );
    }
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
