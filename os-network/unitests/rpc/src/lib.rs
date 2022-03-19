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
use os_network::Factory;
use os_network::MetaFactory;

const DEFAULT_QD_HINT: u64 = 73;

// a test RPC with RDMA
fn test_ud_rpc() {
    log::info!("Test RPC backed by RDMA's UD.");

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
        UDReceiver::new(temp_ud, unsafe { ctx.get_lkey() }),
    );
    rpc_server.get_mut_service().register(73, test_callback);

    log::info!("check RPCHook: {:?}", rpc_server);

    for _ in 0..12 {
        // 64 is the header
        match rpc_server.post_msg_buf(UDMsg::new(4096, 73)) {
            Ok(_) => {}
            Err(e) => log::error!("post recv buf err: {:?}", e),
        }
    }    

    // test RPC request 

    /****************************/
}

#[krdma_test(test_service, test_ud_rpc)]
fn init() {}
