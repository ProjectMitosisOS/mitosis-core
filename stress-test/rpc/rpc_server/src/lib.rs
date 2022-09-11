#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use krdma_test::*;

use os_network::KRdmaKit::comm_manager::CMServer;
use os_network::KRdmaKit::services::UnreliableDatagramAddressService;
use os_network::bytes::*;
use os_network::rpc::*;
use os_network::block_on;
use os_network::timeout::Timeout;
use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::Factory;
use os_network::serialize::Serialize;
use os_network::KRdmaKit;

use KRdmaKit::rdma_shim::linux_kernel_module;
use KRdmaKit::rdma_shim::utils::kthread;
use KRdmaKit::rdma_shim::utils::KTimer;
use KRdmaKit::rdma_shim::bindings::*;
use KRdmaKit::rdma_shim::log;
use KRdmaKit::KDriver;

use rpc_common::random::FastRandom;

use mitosis_macros::{declare_module_param, declare_global};

declare_module_param!(qd_hint, u64);
declare_module_param!(test_rpc_id, usize);
declare_module_param!(running_secs, i64);
declare_module_param!(service_id, u64);

struct WrappedPayload(rpc_common::payload::DefaultSizedPayload);

impl Serialize for WrappedPayload {}

declare_global!(global_random, rpc_common::random::FastRandom);

#[allow(unused_variables)]
fn test_callback(_input: &BytesMut, output: &mut BytesMut) -> usize {
    #[cfg(feature = "checksum-payload")]
    unsafe {
        let payload = WrappedPayload(rpc_common::payload::Payload::create(global_random::get_mut().get_next()));
        payload.serialize(output);
        payload.serialization_buf_len()
    }
    #[cfg(not(feature = "checksum-payload"))]
    0
}

// a test RPC with RDMA
fn start_rpc_server() {
    log::info!("starting rpc server");

    type UDRPCHook<'a> = hook::RPCHook<'a, UDDatagram, UDReceiver, UDFactory>;

    // init RDMA_related data structures
    let driver = unsafe { KDriver::create().unwrap() };
    let ctx = driver.devices().into_iter().next().unwrap().open_context().unwrap();
    let factory = Arc::new(UDFactory::new(&ctx));

    let server_ud = factory.create(
        UDCreationMeta { port: 1 } // Create with default port number 1
    ).unwrap();

    // expose the server-side connection infoit
    // let ctrl = RCtrl::create(service_id::read(), &ctx).unwrap();
    // ctrl.reg_ud(qd_hint::read() as usize, server_ud.get_qp());
    let ud_service = UnreliableDatagramAddressService::create();
    let _server_cm = CMServer::new(service_id::read(), &ud_service, ctx.get_dev_ref());

    // register callback and wait for requests
    let mut rpc_server = UDRPCHook::new(
        factory.clone(),
        server_ud.clone(),
        UDReceiverFactory::new()
            .set_qd_hint(qd_hint::read() as _)
            .create(server_ud.clone()),
    );

    rpc_server
        .get_mut_service()
        .register(test_rpc_id::read(), test_callback);

    log::info!("check RPCHook: {:?}", rpc_server);

    for _ in 0..1024 {
        // 64 is the header
        match rpc_server.post_msg_buf(UDMsg::new(4096, test_rpc_id::read() as u32, server_ud.get_qp().ctx().clone())) {
            Ok(_) => {}
            Err(e) => log::error!("post recv buf err: {:?}", e),
        }
    }

    let timer = KTimer::new();
    let timeout_usec = 1000_000;
    let running_usecs = running_secs::read() * 1_000_000;
    let mut rpc_server = Timeout::new(rpc_server, timeout_usec); // Timeout for 1 second
    let mut rpc_count = 0;
    loop {
        rpc_count += 1;
        if rpc_count % 10000 == 0 {
            kthread::yield_now();
        }
        rpc_server.reset_timer(timeout_usec);
        let res = block_on(&mut rpc_server);
        if res.is_err() {
            if !res.as_ref().err().unwrap().is_elapsed() {
                log::error!(
                    "stress server receiver process err {:?}",
                    res.err().unwrap()
                );
            }
        }
        if timer.get_passed_usec() > running_usecs {
            log::info!("end of rpc server...");
            break;
        }
    }
}

#[krdma_test(start_rpc_server)]
fn init() {
    unsafe {
        global_random::init(FastRandom::new(1));
    }
}
