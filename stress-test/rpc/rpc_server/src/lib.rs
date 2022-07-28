#![no_std]

extern crate alloc;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use rust_kernel_linux_util as log;

use krdma_test::*;
use os_network::bytes::*;
use os_network::rpc::*;
use os_network::block_on;

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util::kthread;
use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util::timer::KTimer;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;
use KRdmaKit::random::FastRandom;

use os_network::timeout::Timeout;
use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::Factory;
use os_network::serialize::Serialize;

use mitosis_macros::{declare_module_param, declare_global};

declare_module_param!(qd_hint, u64);
declare_module_param!(test_rpc_id, usize);
declare_module_param!(running_secs, i64);
declare_module_param!(service_id, u64);

const PAYLOAD_SIZE: usize = 2048;
type SizedPayload = rpc_common::payload::Payload<PAYLOAD_SIZE>;
struct WrappedPayload(SizedPayload);

impl Serialize for WrappedPayload {}

declare_global!(global_random, KRdmaKit::random::FastRandom);

fn test_callback(_input: &BytesMut, output: &mut BytesMut) -> usize {
    unsafe {
        let payload = WrappedPayload(SizedPayload::create(global_random::get_mut().get_next()));
        payload.serialize(output);
        payload.serialization_buf_len()
    }
}

// a test RPC with RDMA
fn start_rpc_server() {
    log::info!("starting rpc server");

    type UDRPCHook<'a, 'b> = hook::RPCHook<'a, 'b, UDDatagram<'a>, UDReceiver<'a>, UDFactory<'a>>;

    // init RDMA_related data structures
    let driver = unsafe { KDriver::create().unwrap() };
    let ctx = driver.devices().into_iter().next().unwrap().open().unwrap();
    let factory = UDFactory::new(&ctx);

    let server_ud = factory.create(()).unwrap();

    // expose the server-side connection infoit
    let ctrl = RCtrl::create(service_id::read(), &ctx).unwrap();
    ctrl.reg_ud(qd_hint::read() as usize, server_ud.get_qp());

    // register callback and wait for requests
    let lkey = unsafe { ctx.get_lkey() };
    let temp_ud = server_ud.clone();
    let mut rpc_server = UDRPCHook::new(
        &factory,
        server_ud,
        UDReceiverFactory::new()
            .set_qd_hint(qd_hint::read() as _)
            .set_lkey(lkey)
            .create(temp_ud),
    );

    rpc_server
        .get_mut_service()
        .register(test_rpc_id::read(), test_callback);

    log::info!("check RPCHook: {:?}", rpc_server);

    for _ in 0..1024 {
        // 64 is the header
        match rpc_server.post_msg_buf(UDMsg::new(4096, test_rpc_id::read() as u32)) {
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
