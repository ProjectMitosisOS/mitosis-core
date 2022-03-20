#![no_std]

extern crate alloc;

use alloc::vec;
use core::fmt::Write;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use rust_kernel_linux_util as log;

use krdma_test::*;
use os_network::bytes::*;
use os_network::rpc::*;

fn test_callback(input: &BytesMut, output: &mut BytesMut) {
    log::info!("test callback input {:?}", input);
    log::info!("test callback output {:?}", output);
}

// a local test
fn test_service() {
    let mut service = Service::new();
    assert_eq!(true, service.register(73, test_callback));
    log::info!("rpc service created! {}", service);

    let mut buf = vec![0; 64];
    let mut msg = unsafe { BytesMut::from_raw(buf.as_mut_ptr(), buf.len()) };
    write!(&mut msg, "hello world").unwrap();

    log::info!("test msg {:?}", msg);

    let mut out_buf = vec![0; 64];
    let mut out_msg = unsafe { BytesMut::from_raw(out_buf.as_mut_ptr(), out_buf.len()) };
    write!(&mut out_msg, "This is the output").unwrap();

    assert_eq!(true, service.execute(73, &mut msg, &mut out_msg));
}

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::rpc::header::*;
use os_network::Factory;
use os_network::MetaFactory;

const DEFAULT_QD_HINT: u64 = 73;
const TEST_RPC_ID: usize = 73;

fn test_rpc_headers() {
    let my_session_id = 21;
    let payload = 14;
    let connect_header = MsgHeader::gen_connect_stub(my_session_id, payload);

    assert!(connect_header.get_connect_stub().is_some());
    assert!(connect_header.get_connect_stub().unwrap().get_session_id() == my_session_id);
}

// a test RPC with RDMA
fn test_ud_rpc() {
    log::info!("Test RPC backed by RDMA's UD.");
    let timeout_usec = 1000_000;

    type UDRPCHook<'a> = hook::RPCHook<'a, UDDatagram<'a>, UDReceiver<'a>>;

    // init RDMA_related data structures
    let driver = unsafe { KDriver::create().unwrap() };
    let nic = driver.devices().into_iter().next().unwrap();
    let factory = UDFactory::new(nic).unwrap();
    let ctx = factory.get_context();

    let server_ud = factory.create(()).unwrap();

    // expose the server-side connection infoit
    let service_id: u64 = 0;
    let ctrl = RCtrl::create(service_id, &ctx).unwrap();
    ctrl.reg_ud(DEFAULT_QD_HINT as usize, server_ud.get_qp());

    // the client part
    let client_ud = factory.create(()).unwrap();
    let gid = ctx.get_gid_as_string();
    let (endpoint, key) = factory
        .create_meta((gid, service_id, DEFAULT_QD_HINT))
        .unwrap();
    log::info!("check endpoint, key: {:?}, {}", endpoint, key);

    let mut client_session = client_ud.create((endpoint, key)).unwrap();

    /**** The main test body****/
    let temp_ud = server_ud.clone();
    let mut rpc_server = UDRPCHook::new(
        server_ud,
        UDReceiverFactory::new()
            .set_qd_hint(DEFAULT_QD_HINT)
            .set_lkey(unsafe { ctx.get_lkey() })
            .create(temp_ud),
    );

    rpc_server
        .get_mut_service()
        .register(TEST_RPC_ID, test_callback);

    log::info!("check RPCHook: {:?}", rpc_server);

    for _ in 0..12 {
        // 64 is the header
        match rpc_server.post_msg_buf(UDMsg::new(4096, 73)) {
            Ok(_) => {}
            Err(e) => log::error!("post recv buf err: {:?}", e),
        }
    }

    use os_network::block_on;
    use os_network::rpc::header::*;
    use os_network::timeout::Timeout;

    // test RPC connect request
    let my_session_id = 73;
    let connect_header = MsgHeader::gen_connect_stub(my_session_id, 64);

    let mut request = UDMsg::new(64, 73);
    unsafe { request.get_bytes_mut().memcpy_serialize(&connect_header) };

    let result = client_session.post(&request, true);
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

    let mut rpc_server = Timeout::new(rpc_server, 10000);
    let res = block_on(&mut rpc_server);
    log::debug!("sanity check result: {:?}", res);
    /****************************/
}

#[krdma_test(test_service, test_rpc_headers, test_ud_rpc)]
fn init() {}
