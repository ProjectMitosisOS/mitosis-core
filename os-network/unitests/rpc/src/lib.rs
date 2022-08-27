#![no_std]

extern crate alloc;

use KRdmaKit::comm_manager::CMServer;
use KRdmaKit::services::UnreliableDatagramAddressService;
use alloc::sync::Arc;
use alloc::vec;
use os_network::UDCreationMeta;
use core::fmt::Write;

use rust_kernel_rdma_base::linux_kernel_module;
use rust_kernel_linux_util as log;

use krdma_test::*;
use os_network::bytes::*;
use os_network::rpc::*;

#[derive(thiserror_no_std::Error, Debug)]
pub enum TestError {
    #[error("Test error {0}")]
    Error(&'static str),
}

fn test_callback(input: &BytesMut, output: &mut BytesMut) -> usize {
    let mut test_val: u64 = 0;
    unsafe { input.memcpy_deserialize(&mut test_val).unwrap() };
    log::info!("test callback input {:?}, decoded {}", input, test_val);
    log::info!("test callback output {:?}", output);
    write!(output, "test_callback").unwrap();
    64
}

// a local test
fn test_service() -> Result<(), TestError> {
    let mut service = Service::new();
    assert_eq!(true, service.register(TEST_RPC_ID, test_callback));
    log::info!("rpc service created! {}", service);

    let mut buf = vec![0; 64];
    let mut msg = unsafe { BytesMut::from_raw(buf.as_mut_ptr(), buf.len()) };
    write!(&mut msg, "hello world").unwrap();

    log::info!("test msg {:?}", msg);

    let mut out_buf = vec![0; 64];
    let mut out_msg = unsafe { BytesMut::from_raw(out_buf.as_mut_ptr(), out_buf.len()) };
    write!(&mut out_msg, "This is the output").unwrap();

    service.execute(TEST_RPC_ID, &mut msg, &mut out_msg).ok_or_else(|| {
        log::error!("Service callback output mismatch.");
        TestError::Error("Callback output error")
    })?;
    Ok(())
}

use rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::rpc::header::*;
use os_network::Factory;
use os_network::MetaFactory;

const DEFAULT_SERVICE_ID: u64 = 37;
const DEFAULT_QD_HINT: u64 = 73;
const CLIENT_QD_HINT: u64 = 12;
const TEST_RPC_ID: usize = 73;
const DEFAULT_PORT: u8 = 1;
const DEFAULT_RECV_BUF_SIZE: usize = 4096;
const DEFAULT_SEND_BUF_SIZE: usize = 1024;
const DEFAULT_SESSION_ID: usize = 74;

// test generation of rpc headers
fn test_rpc_headers() -> Result<(), TestError> {
    let my_session_id = 21;
    let payload = 14;
    let connect_header = MsgHeader::gen_connect_stub(my_session_id, payload);
    let stub = connect_header.get_connect_stub()
        .ok_or_else(|| {
            log::error!("Failed to gen connection stub.");
            TestError::Error("Connect stub gen error.")
        })?;
    if stub.get_session_id() != my_session_id {
        log::error!("Failed to gen correct session id, expected {}, got: {}.", my_session_id, stub.get_session_id());
        Err(TestError::Error("Session id error."))
    } else {
        Ok(())
    }
}

use os_network::block_on;
use os_network::timeout::Timeout;

// a test RPC with RDMA
fn test_ud_rpc() -> Result<(), TestError> {
    log::info!("Test RPC backed by RDMA's UD.");
    let timeout_usec = 5000_000;
    type UDRPCHook<'a> = hook::RPCHook<'a, UDDatagram, UDReceiver, UDFactory>;

    let driver = unsafe { KDriver::create().unwrap() };

    // create Context and UD qp
    let ctx = driver
        .devices()
        .get(0)
        .ok_or(TestError::Error("Not valid device"))?
        .open_context()
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;
    let factory = Arc::new(UDFactory::new(&ctx));
    let server_port = DEFAULT_PORT;
    let client_port = DEFAULT_PORT;
    let server_ud = factory.create(UDCreationMeta {port: server_port}).unwrap();
    let client_ud = factory.create(UDCreationMeta {port: client_port}).unwrap();

    // server side
    // create CMServer and register qp
    let service_id = DEFAULT_SERVICE_ID;
    let ud_server = UnreliableDatagramAddressService::create();
    let _server_cm = CMServer::new(service_id, &ud_server, ctx.get_dev_ref())
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;
    ud_server.reg_qp(DEFAULT_QD_HINT as usize, &server_ud.get_qp());
    ud_server.reg_qp(CLIENT_QD_HINT as usize, &client_ud.get_qp());

    // client side
    // get endpoint meta from remote
    let gid = ctx.get_dev_ref().query_gid(server_port, 0).unwrap();
    let meta = UDHyperMeta {
        gid,
        service_id,
        qd_hint: DEFAULT_QD_HINT as usize,
        local_port: client_port,
    };
    let endpoint = factory.create_meta(meta).map_err(|_| {
        log::error!("Create endpoint error.");
        TestError::Error("Endpoint error.")
    })?;

    // create client-side session and receiver
    let mut client_session = client_ud.create(endpoint).unwrap();
    let mut client_receiver = UDReceiverFactory::new()
        .set_qd_hint(CLIENT_QD_HINT as _)
        .create(client_ud);

    // the main test body
    let temp_ud = server_ud.clone();
    let mut rpc_server = UDRPCHook::new(
        factory,
        server_ud,
        UDReceiverFactory::new()
            .set_qd_hint(DEFAULT_QD_HINT as _)
            .create(temp_ud),
    );

    rpc_server
        .get_mut_service()
        .register(TEST_RPC_ID, test_callback);

    log::info!("check RPCHook: {:?}", rpc_server);

    for _ in 0..12 {
        // 64 is the header
        match rpc_server.post_msg_buf(UDMsg::new(DEFAULT_RECV_BUF_SIZE, 0, ctx.clone())) {
            Ok(_) => {}
            Err(e) => log::error!("post recv buf err: {:?}", e),
        }
        client_receiver.post_recv_buf(UDMsg::new(DEFAULT_RECV_BUF_SIZE, 0, ctx.clone())).unwrap(); // should succeed
    }

    // test RPC connect request
    let my_session_id = DEFAULT_SESSION_ID;

    let mut request = UDMsg::new(DEFAULT_SEND_BUF_SIZE, 1, ctx.clone());
    let req_sz = os_network::rpc::ConnectStubFactory::new(my_session_id)
        .generate(
            &UDHyperMeta {
                gid,
                service_id,
                qd_hint: CLIENT_QD_HINT as usize,
                local_port: client_port,
            },
            request.get_bytes_mut(),
        )
        .unwrap();
    
    log::debug!("check request size: {}", req_sz);
    log::debug!("sanity check connect stub {:?}", unsafe {
        request.get_bytes().clone_and_resize(64) // TODO
    });

    // send connect request at client side
    client_session.post(&request, req_sz, true)
        .map_err(|_| {
            log::error!("Client session post error.");
            TestError::Error("Session post error.")
        })?;

    // check the message has been sent
    let mut timeout_client = Timeout::new(client_session, timeout_usec);
    block_on(&mut timeout_client)
        .map_err(|_| {
            log::error!("Client session poll error.");
            TestError::Error("Session poll error.")
        })?;

    log::info!("client side post msg done");

    // run the server event loop to receive message
    let mut rpc_server = Timeout::new(rpc_server, timeout_usec);
    block_on(&mut rpc_server)
        .map_err(|e| {
            log::error!("Server receiver process err {:?}", e);
            TestError::Error("Server receiver error.")
        })?;

    // check the reply at client side
    let mut client_receiver = Timeout::new(client_receiver, timeout_usec);
    let res = block_on(&mut client_receiver)
        .map_err(|e| {
            log::error!("Client receiver reply error {:?}.", e);
            TestError::Error("Client receiver error.")
        })?;
    let bytes = unsafe { res.get_bytes().clone() };
    let msg_header_bytes = unsafe { bytes.truncate_header(UDReceiver::HEADER).unwrap() };
    let mut msg_header: MsgHeader = Default::default();
    unsafe { msg_header_bytes.memcpy_deserialize(&mut msg_header) };
    log::info!("Sanity check decoded reply {:?}", msg_header);

    // send another request
    let mut client_session = timeout_client.into_inner();
    let req_sz = os_network::rpc::CallStubFactory::new(my_session_id, TEST_RPC_ID)
        .generate(&(666 as u64), request.get_bytes_mut()) // 666 is a dummy RPC argument
        .unwrap();
    client_session.post(&request, req_sz, false)
        .map_err(|_| {
            log::error!("Client session post error.");
            TestError::Error("Session post error.")
        })?;

    // poll the RPC server
    rpc_server.reset_timer(timeout_usec);
    block_on(&mut rpc_server)
        .map_err(|e| {
            log::error!("Server receiver process err {:?}", e);
            TestError::Error("Server receiver error.")
        })?;
        
    // check the reply at client side
    client_receiver.reset_timer(timeout_usec);
    let res = block_on(&mut client_receiver)
        .map_err(|e| {
            log::error!("Client receiver reply error {:?}.", e);
            TestError::Error("Client receiver error.")
        })?;
    let bytes = unsafe { res.get_bytes().clone() };
    let msg_header_bytes = unsafe { bytes.truncate_header(UDReceiver::HEADER).unwrap() };
    let mut msg_header: MsgHeader = Default::default();
    unsafe { msg_header_bytes.memcpy_deserialize(&mut msg_header) };
    log::info!("sanity check decoded reply {:?}", msg_header);

    // finally check the rpc server status
    let rpc_server = rpc_server.into_inner();
    log::debug!("final check hook status {:?}", rpc_server);
    Ok(())
}

use os_network::rpc::impls::ud::UDSession;

fn test_ud_rpc_elegant() -> Result<(), TestError> {
    log::info!("Test RPC backed by RDMA's UD with elegant wrapper.");
    let timeout_usec = 5000_000;
    type UDRPCHook<'a> = hook::RPCHook<'a, UDDatagram, UDReceiver, UDFactory>;
    type UDCaller = Caller<UDReceiver, UDSession>;

    let driver = unsafe { KDriver::create().unwrap() };

    // create Context and UD qp
    let ctx = driver
        .devices()
        .get(0)
        .ok_or(TestError::Error("Not valid device"))?
        .open_context()
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;
    let factory = Arc::new(UDFactory::new(&ctx));
    let server_port = DEFAULT_PORT;
    let client_port = DEFAULT_PORT;
    let server_ud = factory.create(UDCreationMeta {port: server_port}).unwrap();
    let client_ud = factory.create(UDCreationMeta {port: client_port}).unwrap();

    // server side
    // create CMServer and register qp
    let service_id = DEFAULT_SERVICE_ID;
    let ud_server = UnreliableDatagramAddressService::create();
    let _server_cm = CMServer::new(service_id, &ud_server, ctx.get_dev_ref())
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;
    ud_server.reg_qp(DEFAULT_QD_HINT as usize, &server_ud.get_qp());
    ud_server.reg_qp(CLIENT_QD_HINT as usize, &client_ud.get_qp());

    // client side
    // get endpoint meta from remote
    let gid = ctx.get_dev_ref().query_gid(server_port, 0).unwrap();
    let meta = UDHyperMeta {
        gid,
        service_id,
        qd_hint: DEFAULT_QD_HINT as usize,
        local_port: client_port,
    };
    let endpoint = factory.create_meta(meta).map_err(|_| {
        log::error!("Create endpoint error.");
        TestError::Error("Endpoint error.")
    })?;

    // create client-side session and receiver
    let client_session = client_ud.create(endpoint).unwrap();
    let mut client_receiver = UDReceiverFactory::new()
        .set_qd_hint(CLIENT_QD_HINT as _)
        .create(client_ud);

    // the main test body
    let temp_ud = server_ud.clone();
    let mut rpc_server = UDRPCHook::new(
        factory,
        server_ud,
        UDReceiverFactory::new()
            .set_qd_hint(DEFAULT_QD_HINT as _)
            .create(temp_ud),
    );

    rpc_server
        .get_mut_service()
        .register(TEST_RPC_ID, test_callback);

    log::info!("check RPCHook: {:?}", rpc_server);

    for _ in 0..12 {
        // 64 is the header
        match rpc_server.post_msg_buf(UDMsg::new(DEFAULT_RECV_BUF_SIZE, 0, ctx.clone())) {
            Ok(_) => {}
            Err(e) => log::error!("post recv buf err: {:?}", e),
        }
        client_receiver.post_recv_buf(UDMsg::new(DEFAULT_RECV_BUF_SIZE, 0, ctx.clone())).unwrap(); // should succeed
    }
    
    // client
    let my_session_id = DEFAULT_SESSION_ID;
    let mut caller = UDCaller::new(client_receiver);
    caller
        .connect(
            my_session_id,
            my_session_id, 
            client_session,
            UDHyperMeta {
                gid,
                service_id: service_id,
                qd_hint: CLIENT_QD_HINT as usize,
                local_port: client_port,
            },
        ).map_err(|_| {
            log::error!("Client caller connect error.");
            TestError::Error("Caller connect error.")
        })?;

    // server first run the event loop to receive the connect message
    let mut rpc_server = Timeout::new(rpc_server, timeout_usec);
    block_on(&mut rpc_server)
        .map_err(|e| {
            log::error!("Server receiver process err {:?}", e);
            TestError::Error("Server receiver error.")
        })?;

    // then, the client can check the connection result
    let mut caller_timeout = Timeout::new(caller, timeout_usec);
    let res = block_on(&mut caller_timeout)
        .map_err(|e| {
            log::error!("Client receiver process err {:?}", e);
            TestError::Error("Client receiver error.")
        })?;
    log::debug!("sanity check client connection result: {:?}", res.1);

    // client makes another simple call
    let mut caller = caller_timeout.into_inner();
    caller
        .sync_call(
            my_session_id,
            my_session_id,
            TEST_RPC_ID,
            666 as u64 // 666 is a dummy RPC argument
        ).map_err(|_| {
            log::error!("Client caller call rpc error.");
            TestError::Error("Caller caller error.")
        })?;

    // receive at the server
    rpc_server.reset_timer(timeout_usec);
    block_on(&mut rpc_server)
        .map_err(|e| {
            log::error!("Server receiver process err {:?}", e);
            TestError::Error("Server receiver error.")
        })?;

    // then, the client can check the rpc result
    let mut caller_timeout = Timeout::new(caller, timeout_usec);
    let res = block_on(&mut caller_timeout)
        .map_err(|e| {
            log::error!("Client receiver process err {:?}", e);
            TestError::Error("Client receiver error.")
        })?;
    log::debug!("sanity check client rpc result: {:?}", res.1);

    let rpc_server = rpc_server.into_inner();
    log::debug!("final check hook status {:?}", rpc_server);
    Ok(())    
}

fn test_wrapper() -> Result<(), TestError> {
    test_service()?;
    test_rpc_headers()?;
    test_ud_rpc()?;
    test_ud_rpc_elegant()?;
    Ok(())
}

#[krdma_main]
fn main() {
    match test_wrapper() {
        Ok(_) => {
            log::info!("pass all tests")
        }
        Err(e) => {
            log::error!("test error {:?}", e)
        }
    };
}
