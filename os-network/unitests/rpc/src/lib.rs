#![no_std]

extern crate alloc;

use alloc::vec;
use core::fmt::Write;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use rust_kernel_linux_util as log;

use krdma_test::*;
use os_network::bytes::*;
use os_network::rpc::*;

fn test_callback(input: &BytesMut, output: &mut BytesMut) -> usize {
    let mut test_val: u64 = 0;
    unsafe { input.memcpy_deserialize(&mut test_val).unwrap() };
    log::info!("test callback input {:?}, decoded {}", input, test_val);
    log::info!("test callback output {:?}", output);
    write!(output, "test_callback").unwrap();
    64
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

    assert_eq!(true, service.execute(73, &mut msg, &mut out_msg).is_some());
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
const CLIENT_QD_HINT: u64 = 12;
const TEST_RPC_ID: usize = 73;

fn test_rpc_headers() {
    let my_session_id = 21;
    let payload = 14;
    let connect_header = MsgHeader::gen_connect_stub(my_session_id, payload);

    assert!(connect_header.get_connect_stub().is_some());
    assert!(connect_header.get_connect_stub().unwrap().get_session_id() == my_session_id);
}

use os_network::block_on;
use os_network::timeout::Timeout;

// a test RPC with RDMA
fn test_ud_rpc() {
    log::info!("Test RPC backed by RDMA's UD.");
    let timeout_usec = 1000_000;

    type UDRPCHook<'a, 'b> = hook::RPCHook<'a, 'b, UDDatagram<'a>, UDReceiver<'a>, UDFactory<'a>>;

    // init RDMA_related data structures
    let driver = unsafe { KDriver::create().unwrap() };
    let nic = driver.devices().into_iter().next().unwrap();
    let factory = UDFactory::new(nic).unwrap();
    let ctx = factory.get_context();

    let server_ud = factory.create(()).unwrap();
    let client_ud = factory.create(()).unwrap();

    // expose the server-side connection infoit
    let service_id: u64 = 0;
    let ctrl = RCtrl::create(service_id, &ctx).unwrap();
    ctrl.reg_ud(DEFAULT_QD_HINT as usize, server_ud.get_qp());
    ctrl.reg_ud(CLIENT_QD_HINT as usize, client_ud.get_qp());

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

    let lkey = unsafe { ctx.get_lkey() };

    let mut client_session = client_ud.create((endpoint, key)).unwrap();
    let mut client_receiver = UDReceiverFactory::new()
        .set_qd_hint(CLIENT_QD_HINT as _)
        .set_lkey(lkey)
        .create(client_ud);

    /**** The main test body****/
    let temp_ud = server_ud.clone();
    let mut rpc_server = UDRPCHook::new(
        &factory,
        server_ud,
        UDReceiverFactory::new()
            .set_qd_hint(DEFAULT_QD_HINT as _)
            .set_lkey(lkey)
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
        client_receiver.post_recv_buf(UDMsg::new(4096, 73)).unwrap(); // should succeed
    }

    // test RPC connect request
    let my_session_id = 73;

    let mut request = UDMsg::new(1024, 73);
    let req_sz = os_network::rpc::ConnectStubFactory::new(my_session_id)
        .generate(
            &UDHyperMeta {
                gid: os_network::rdma::RawGID::new(ctx.get_gid_as_string()).unwrap(),
                service_id: service_id,
                qd_hint: CLIENT_QD_HINT,
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

    // run the server event loop to receive message
    let mut rpc_server = Timeout::new(rpc_server, 10000);
    let res = block_on(&mut rpc_server);
    log::debug!("sanity check result: {:?}", res);

    // check the client reply
    let mut client_receiver = Timeout::new(client_receiver, 10000);
    let res = block_on(&mut client_receiver);
    match res {
        Ok(msg) => {
            let bytes = unsafe { msg.get_bytes().clone() };
            let msg_header_bytes = unsafe { bytes.truncate_header(UDReceiver::HEADER).unwrap() };
            let mut msg_header: MsgHeader = Default::default();
            unsafe { msg_header_bytes.memcpy_deserialize(&mut msg_header) };
            log::info!("sanity check decoded reply {:?}", msg_header);
        }
        Err(e) => log::error!("client receiver reply err {:?}", e),
    }

    // send another request
    let mut client_session = timeout_client.into_inner();
    let req_sz = os_network::rpc::CallStubFactory::new(my_session_id, 73)
        .generate(&(73 as u64), request.get_bytes_mut()) // 0 is a dummy RPC argument
        .unwrap();
    let _result = client_session.post(&request, req_sz, false);

    // poll the RPC completions
    rpc_server.reset_timer(1000_000);
    let res = block_on(&mut rpc_server);
    if res.is_err() {
        log::error!("server receiver process err {:?}", res.err().unwrap());
    }

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
        }
    }

    /****************************/

    let rpc_server = rpc_server.into_inner();
    log::debug!("final check hook status {:?}", rpc_server);
}

use os_network::rpc::impls::ud::UDSession;

fn test_ud_rpc_elegant() {
    log::info!("Test RPC backed by RDMA's UD with elegant wrapper.");
    let timeout_usec = 1000_000;

    type UDRPCHook<'a, 'b> = hook::RPCHook<'a, 'b, UDDatagram<'a>, UDReceiver<'a>, UDFactory<'a>>;
    type UDCaller<'a> = Caller<UDReceiver<'a>, UDSession<'a>>;

    /*********Driver part****************/
    let driver = unsafe { KDriver::create().unwrap() };
    let nic = driver.devices().into_iter().next().unwrap();
    let factory = UDFactory::new(nic).unwrap();
    let ctx = factory.get_context();

    let service_id: u64 = 0;
    let ctrl = RCtrl::create(service_id, &ctx).unwrap();
    /*********Driver part done***********/

    /*********Raw connection part****************/
    let server_ud = factory.create(()).unwrap();
    let client_ud = factory.create(()).unwrap();

    ctrl.reg_ud(DEFAULT_QD_HINT as usize, server_ud.get_qp());
    ctrl.reg_ud(CLIENT_QD_HINT as usize, client_ud.get_qp());

    let gid = os_network::rdma::RawGID::new(ctx.get_gid_as_string()).unwrap();

    let (endpoint, key) = factory
        .create_meta(UDHyperMeta {
            gid: gid,
            service_id: service_id,
            qd_hint: DEFAULT_QD_HINT,
        })
        .unwrap();
    let lkey = unsafe { ctx.get_lkey() };
    /*********Raw connection part done***********/

    /*********RPC data structure support  ****************/
    // client
    let client_session = client_ud.create((endpoint, key)).unwrap();
    let client_receiver = UDReceiverFactory::new()
        .set_qd_hint(CLIENT_QD_HINT as _)
        .set_lkey(lkey)
        .create(client_ud);
    /*********RPC data structure support done***********/

    /*********RPC main data structure****************/
    // server
    let temp_ud = server_ud.clone();
    let mut rpc_server = UDRPCHook::new(
        &factory,
        server_ud,
        UDReceiverFactory::new()
            .set_qd_hint(DEFAULT_QD_HINT as _)
            .set_lkey(lkey)
            .create(temp_ud),
    );
    rpc_server
        .get_mut_service()
        .register(TEST_RPC_ID, test_callback);

    // client
    let mut caller = UDCaller::new(client_receiver);

    // pre-most receive buffers
    for _ in 0..12 {
        // 64 is the header
        match rpc_server.post_msg_buf(UDMsg::new(4096, 73)) {
            Ok(_) => {}
            Err(e) => log::error!("post recv buf err: {:?}", e),
        }
        caller.register_recv_buf(UDMsg::new(4096, 73)).unwrap(); // should succeed
    }

    // client
    caller
        .connect(
            73,
            client_session,
            UDHyperMeta {
                gid: os_network::rdma::RawGID::new(ctx.get_gid_as_string()).unwrap(),
                service_id: service_id,
                qd_hint: CLIENT_QD_HINT,
            },
        )
        .unwrap();
    /*********RPC main data structure done***********/

    /*********Main test body***********/

    // server first run the event loop to receive the connect message
    let mut rpc_server = Timeout::new(rpc_server, timeout_usec);
    let res = block_on(&mut rpc_server);
    log::debug!("sanity check server handle connect result: {:?}", res);

    //  then, the client can check the result
    let mut caller_timeout = Timeout::new(caller, timeout_usec);
    let res = block_on(&mut caller_timeout);
    match res {
        Ok(v) => {
            let (_, reply) = v;
            log::debug!("sanity check client connection result: {:?}", reply);
        }
        Err(e) => log::error!("client connect error: {:?}", e),
    };

    let mut caller = caller_timeout.into_inner();

    // client make a simple call    
    caller.sync_call(73,TEST_RPC_ID, 128 as u64).unwrap(); 
    
    // receive at the server
    rpc_server.reset_timer(timeout_usec);
    let res = block_on(&mut rpc_server); 
    log::debug!("sanity check server handle call result: {:?}", res);
    
    // receive the client reply
    let mut caller_timeout = Timeout::new(caller, timeout_usec);
    let res = block_on(&mut caller_timeout);
    match res {
        Ok(v) => {
            let (_, reply) = v;
            log::debug!("sanity check client call result: {:?}", reply);
        }
        Err(e) => log::error!("client call error: {:?}", e),
    };    

    /*********Main test body done***********/
    let rpc_server = rpc_server.into_inner();
    log::debug!("final check hook status {:?}", rpc_server);    
}

#[krdma_test(test_service, test_rpc_headers, test_ud_rpc, test_ud_rpc_elegant)]
fn init() {}
