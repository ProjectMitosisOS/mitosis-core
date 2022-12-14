#![no_std]

extern crate alloc;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use rust_kernel_linux_util as log;

use krdma_test::*;
use os_network::bytes::*;
use os_network::rpc::*;

fn test_callback(_input: &BytesMut, _output: &mut BytesMut) -> usize {
    0
}

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util::kthread;
use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util::timer::KTimer;
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

// a test RPC with RDMA
fn test_ud_rpc() {
    log::info!("Test RPC backed by RDMA's UD.");
    let timeout_usec = 1000_000;

    type UDRPCHook<'a, 'b> = hook::RPCHook<'a, 'b, UDDatagram<'a>, UDReceiver<'a>, UDFactory<'a>>;

    // init RDMA_related data structures
    let driver = unsafe { KDriver::create().unwrap() };
    let ctx = driver.devices().into_iter().next().unwrap().open().unwrap();
    let factory = UDFactory::new(&ctx);

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

    use os_network::block_on;
    use os_network::timeout::Timeout;

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
    let mut client_session = timeout_client.into_inner();

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

    // make a stress test
    let stress_count = 10_000_000;
    client_receiver.reset_timer(20_000);

    // first, test the signaled performance
    let timer = KTimer::new();
    for i in 0..stress_count {
        if i % 10000 == 0 {
            // prevent softlock
            kthread::yield_now();
        }
        let req_sz = os_network::rpc::CallStubFactory::new(my_session_id, 73)
            .generate(&(0 as u64), request.get_bytes_mut()) // 0 is a dummy RPC argument
            .unwrap();

        let result = client_session.post(&request, req_sz, true);
        if result.is_err() {
            log::error!("fail to post message in a stress test");
            break;
        }
        // check the message has been sent
        let mut timeout_client = Timeout::new(client_session, timeout_usec);
        block_on(&mut timeout_client).unwrap();
        client_session = timeout_client.into_inner();

        // poll the RPC completions
        rpc_server.reset_timer(1000_000);
        let res = block_on(&mut rpc_server);
        if res.is_err() {
            log::error!(
                "stress server receiver process err {:?}",
                res.err().unwrap()
            );
            break;
        }
        client_receiver.reset_timer(1000_000);
        let res = block_on(&mut client_receiver);
        match res {
            Ok(msg) => {
                client_receiver.get_inner_mut().post_recv_buf(msg).unwrap();
            }
            Err(e) => {
                log::error!("stress client receiver reply err {:?}", e);
                break;
            }
        }
    }
    log::info!(
        "[Pass #1 done] passed {} secs",
        timer.get_passed_usec() / 1000000
    );

    client_receiver.reset_timer(20_000);

    // then, test the un-signaled performance
    let timer = KTimer::new();
    for i in 0..stress_count {
        if i % 10000 == 0 {
            // prevent softlock
            kthread::yield_now();
        }
        let req_sz = os_network::rpc::CallStubFactory::new(my_session_id, 73)
            .generate(&(0 as u64), request.get_bytes_mut()) // 0 is a dummy RPC argument
            .unwrap();

        let result = if client_session.get_pending_reqs() == 0 {
            client_session.post(&request, req_sz, true)
        } else {
            client_session.post(&request, req_sz, false)
        };
        if result.is_err() {
            log::error!("fail to post message in a stress test");
            break;
        }

        if client_session.get_pending_reqs() >= 16 {
            block_on(&mut client_session).unwrap();
        }

        // poll the RPC completions
        rpc_server.reset_timer(1000_000);
        let res = block_on(&mut rpc_server);
        if res.is_err() {
            log::error!(
                "stress server receiver process err {:?}",
                res.err().unwrap()
            );
            break;
        }
        client_receiver.reset_timer(1000_000);
        let res = block_on(&mut client_receiver);
        match res {
            Ok(msg) => {
                client_receiver.get_inner_mut().post_recv_buf(msg).unwrap();
            }
            Err(e) => {
                log::error!("stress client receiver reply err {:?}", e);
                break;
            }
        }
    }
    log::info!(
        "[Pass #2 done] passed {} secs",
        timer.get_passed_usec() / 1000000
    );

    /****************************/
    let rpc_server = rpc_server.into_inner();
    log::debug!("final check hook status {:?}", rpc_server);

    if rpc_server.get_analysis().get_ncalls() != stress_count * 2 {
        log::error!(
            "failed to handle all the stress calls {}",
            rpc_server.get_analysis().get_ncalls()
        );
    }
}

#[krdma_test(test_ud_rpc)]
fn init() {}
