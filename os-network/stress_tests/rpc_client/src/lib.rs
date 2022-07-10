#![no_std]

extern crate alloc;
use alloc::format;
use alloc::vec::Vec;
use rust_kernel_linux_util::string::ptr2string;
use rust_kernel_linux_util::linux_kernel_module::c_types::c_void;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use rust_kernel_linux_util as log;

use krdma_test::*;
use os_network::bytes::*;
use os_network::rpc::*;
use os_network::block_on;

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util::kthread;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use os_network::timeout::Timeout;
use os_network::rpc::header::MsgHeader;
use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::Factory;
use os_network::MetaFactory;

use mitosis_macros::{declare_module_param, declare_global};

declare_module_param!(server_qd_hint, u64);
declare_module_param!(client_qd_hint, u64);
declare_module_param!(server_service_id, u64);
declare_module_param!(client_service_id_base, u64);
declare_module_param!(session_id_base, usize);
declare_module_param!(thread_count, u64);
declare_module_param!(test_rpc_id, u32);
declare_module_param!(running_secs, i64);
declare_module_param!(gid, *mut u8);

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);

extern "C" fn stress_test_routine(id: *mut c_void) -> i32 {
    let id = id as u64;
    let driver = unsafe { KDRIVER::get_ref() };
    let server_gid_str = unsafe { ptr2string(gid::read()) };
    let client_service_id = client_service_id_base::read() + id;
    let my_session_id = session_id_base::read() + id as usize;
    let timeout_usec = 1000_000;
    let mut is_error = false;
    log::info!("start stress test client {}", id);
    log::info!("gid: {}", server_gid_str);

    let ctx = driver.devices().into_iter().next().unwrap().open().unwrap();
    let factory = UDFactory::new(&ctx);
    let client_ud = factory.create(()).unwrap();

    let ctrl = RCtrl::create(client_service_id, &ctx).unwrap();
    ctrl.reg_ud(client_qd_hint::read() as usize, client_ud.get_qp());

    let server_gid = os_network::rdma::RawGID::new(server_gid_str).unwrap();
    let (endpoint, key) = factory
        .create_meta(UDHyperMeta {
            gid: server_gid,
            service_id: server_service_id::read(),
            qd_hint: server_qd_hint::read(),
        })
        .unwrap();
    log::info!("check endpoint, key: {:?}, {}", endpoint, key);

    let lkey = unsafe { ctx.get_lkey() };
    let mut client_session = client_ud.create((endpoint, key)).unwrap();
    let mut client_receiver = UDReceiverFactory::new()
        .set_qd_hint(client_qd_hint::read() as _)
        .set_lkey(lkey)
        .create(client_ud);

    for _ in 0..12 {
        client_receiver.post_recv_buf(UDMsg::new(4096, test_rpc_id::read())).unwrap();
    }

    // rpc connection request
    let mut request = UDMsg::new(1024, test_rpc_id::read());
    let req_sz = os_network::rpc::ConnectStubFactory::new(my_session_id)
        .generate(
            &UDHyperMeta {
                gid: os_network::rdma::RawGID::new(ctx.get_gid_as_string()).unwrap(),
                service_id: client_service_id,
                qd_hint: client_qd_hint::read(),
            },
            request.get_bytes_mut(),
        )
        .unwrap();

    log::debug!("sanity check connect stub {:?}", unsafe {
        request.get_bytes().clone_and_resize(64)
    });

    let result = client_session.post(&request, req_sz, true);
    if result.is_err() {
        log::error!("fail to post message");
        return 0;
    }

    // check the message has been sent
    let mut timeout_client = Timeout::new(client_session, timeout_usec);
    let result = block_on(&mut timeout_client);
    let mut client_session = timeout_client.into_inner();

    if result.is_err() {
        log::error!("polling send ud qp with error: {:?}", result.err().unwrap());
    } else {
        log::info!("post msg done");
    }

    // check the client reply
    let mut client_receiver = Timeout::new(client_receiver, timeout_usec);
    let res = block_on(&mut client_receiver);
    match res {
        Ok(msg) => {
            let bytes = unsafe { msg.get_bytes().clone() };
            let msg_header_bytes = unsafe { bytes.truncate_header(UDReceiver::HEADER).unwrap() };
            let mut msg_header: MsgHeader = Default::default();
            unsafe { msg_header_bytes.memcpy_deserialize(&mut msg_header) };
            log::info!("sanity check decoded reply {:?}", msg_header);
        }
        Err(e) => {
            log::error!("client receiver reply err {:?}", e);
            is_error = true;
        },
    }

    // start stress test
    let mut count = 0;
    client_receiver.reset_timer(timeout_usec);
    while !kthread::should_stop() {
        count += 1;
        if count % 10000 == 0 {
            // prevent softlock
            kthread::yield_now();
        }
        if is_error { continue; }
        let req_sz = os_network::rpc::CallStubFactory::new(my_session_id, test_rpc_id::read() as usize)
            .generate(&(0 as u64), request.get_bytes_mut()) // 0 is a dummy RPC argument
            .unwrap();
        let result = client_session.post(&request, req_sz, true);
        if result.is_err() {
            log::error!("fail to post message in a stress test");
            is_error = true;
            continue;
        }
        // check the message has been sent
        let mut timeout_client = Timeout::new(client_session, timeout_usec);
        block_on(&mut timeout_client).unwrap();
        client_session = timeout_client.into_inner();
        // check the client receiver
        client_receiver.reset_timer(timeout_usec);
        let res = block_on(&mut client_receiver);
        match res {
            Ok(msg) => {
                client_receiver.get_inner_mut().post_recv_buf(msg).unwrap();
            }
            Err(e) => {
                log::error!("#{} stress client receiver reply err {:?}", id, e);
                is_error = true;
                continue;
            }
        }
    }
    0
}

// start multiple rpc client threads
fn start_rpc_client() {
    log::info!("starting rpc client");
    
    // spawn multiple threads
    let mut handlers = Vec::new();
    for i in 0..thread_count::read() {
        let name = format!("rpc client {}", i);
        let builder = kthread::Builder::new()
                        .set_name(name)
                        .set_parameter(i as *mut c_void);
        let handler = builder.spawn(stress_test_routine).unwrap();
        handlers.push(handler);
    }

    kthread::sleep(running_secs::read() as u32);

    // stop all the threads
    while let Some(handler) = handlers.pop() {
        handler.join();
    }
    log::info!("end of rpc client...");
}

#[krdma_test(start_rpc_client)]
fn init() {
    unsafe {
        KDRIVER::init(KDriver::create().unwrap());
    }
}

#[krdma_drop]
fn drop() {
    unsafe {
        KDRIVER::drop();
    }
}
