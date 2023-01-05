#![no_std]

extern crate alloc;

use mitosis::KRdmaKit::comm_manager::CMServer;
use mitosis::KRdmaKit::services::UnreliableDatagramAddressService;
use mitosis::linux_kernel_module;
use mitosis::log;
use mitosis::rust_kernel_linux_util::kthread;

use mitosis::startup::*;
use mitosis::os_network;

use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::rpc::impls::ud::UDSession;
use os_network::rpc::*;

use os_network::Factory;
use os_network::MetaFactory;

use os_network::block_on;
use os_network::timeout::Timeout;

use krdma_test::*;

// Note, test_rpc_two should be called after `test_rpc`
fn test_rpc_two() {
    log::info!("Test RPC Two using the refined API.");

    let pool_idx = 0;

    // it is ok, because in the unittest, all sender & receiver share the same context
    let context = unsafe {
        mitosis::get_rpc_caller_pool_ref()
            .get_caller_context(pool_idx)
            .unwrap()
    };

    let _ = unsafe { mitosis::get_rpc_caller_pool_mut() }
        .connect_session_at(
            pool_idx,
            0xdeadbeaf, // Notice: it is very important to ensure that session ID is unique!
            0xdeadbeaf,
            UDHyperMeta {
                // the remote machine's RDMA gid. Since we are unit test, we use the local gid (port=1, gid_idx=0)
                gid: context.query_gid(1, 0).unwrap(),

                // CM server on RNIC 0 listens on mitosis::rdma_context::SERVICE_ID_BASE,
                // CM server on RNIC 1 listens on mitosis::rdma_context::SERVICE_ID_BASE + 1, etc
                service_id: mitosis::rdma_context::SERVICE_ID_BASE,

                // Thread 0's UD registers on mitosis::rpc_service::QD_HINT_BASE,
                // Thread 1's UD registers on mitosis::rpc_service::QD_HINT_BASE + 1, etcc
                qd_hint: (mitosis::rpc_service::QD_HINT_BASE) as _,

                ..Default::default()
            },
        )
        .expect("failed to connect the endpoint");

    // now we can call!
    let caller = unsafe {
        mitosis::rpc_caller_pool::CallerPool::get_global_caller(pool_idx)
            .expect("the caller should be properly inited")
    };

    crate::log::info!("now start to test RPC caller");

    caller.lock(|caller| {
        caller
            .sync_call(
                0xdeadbeaf,                              // remote session ID
                0xdeadbeaf,                              // local session ID
                mitosis::rpc_handlers::RPCId::Echo as _, // RPC ID
                0xffffffff as u64,                       // send an arg of u64
            )
            .unwrap();

        let res = block_on(caller);
        match res {
            Ok(v) => {
                let (_, reply) = v; // msg, reply
                log::debug!("sanity check rpc two call result: {:?}", reply);
                // Note that in real benchmark, we need to register the _ to the caller
            }
            Err(e) => log::error!("client call error: {:?}", e),
        };
    })
}

fn test_rpc() {
    type UDCaller = Caller<UDReceiver, UDSession>;

    log::info!("in test rpc");

    // we need to sleep 1 second,
    // to ensure that the RPC servers has successfully created, since they are created in an async way

    kthread::sleep(1);

    let factory = unsafe { mitosis::get_ud_factory_ref(0).expect("no available factory") };
    let context = factory.get_context();

    // init caller
    let client_ud = factory.create(
        UDCreationMeta { port: 1 }
    ).unwrap();

    // use a disjoint service ID different than the server
    let my_service_id = mitosis::rdma_context::SERVICE_ID_BASE - 1;

    // let ctrl = RCtrl::create(mitosis::rdma_context::SERVICE_ID_BASE - 1, context).unwrap();
    // ctrl.reg_ud(0 as usize, client_ud.get_qp());
    let service = UnreliableDatagramAddressService::create();
    let _cm_server = CMServer::new(my_service_id, &service, context.get_dev_ref());
    service.reg_qp(0 as usize, &client_ud.get_qp());

    let endpoint = factory
        .create_meta(UDHyperMeta {
            gid: context.query_gid(1, 0).unwrap(),
            service_id: mitosis::rdma_context::SERVICE_ID_BASE,
            qd_hint: mitosis::rpc_service::QD_HINT_BASE as _,
            ..Default::default()
        })
        .unwrap();

    let endpoint1 = factory
        .create_meta(UDHyperMeta {
            gid: context.query_gid(1, 0).unwrap(),
            service_id: mitosis::rdma_context::SERVICE_ID_BASE,
            qd_hint: (mitosis::rpc_service::QD_HINT_BASE + 1) as _,
            ..Default::default()
        })
        .unwrap();

    let client_session = client_ud.create(endpoint).unwrap();
    let client_session_1 = client_ud.create(endpoint1).unwrap();

    let client_receiver = UDReceiverFactory::new()
        .set_qd_hint(0 as _)
        .create(client_ud);

    let mut caller = UDCaller::new(client_receiver);
    for _ in 0..12 {
        caller.register_recv_buf(UDMsg::new(4096, 0, context.clone())).unwrap(); // should succeed
    }

    caller
        .connect(
            73,
            73,
            client_session,
            UDHyperMeta {
                gid: context.query_gid(1, 0).unwrap(),
                service_id: mitosis::rdma_context::SERVICE_ID_BASE - 1,
                qd_hint: 0,
                ..Default::default()
            },
        )
        .unwrap();

    let timeout_usec = 1000_000; // 1 sec
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

    // now call two RPCs
    let mut caller = caller_timeout.into_inner();
    caller
        .sync_call(73, 73, mitosis::rpc_handlers::RPCId::Nil as _, 128 as u64)
        .unwrap();
    let mut caller_timeout = Timeout::new(caller, timeout_usec);
    let res = block_on(&mut caller_timeout);
    match res {
        Ok(v) => {
            let (_, reply) = v;
            log::debug!("sanity check client call result: {:?}", reply);
        }
        Err(e) => log::error!("client call error: {:?}", e),
    };

    let mut caller = caller_timeout.into_inner();
    caller
        .sync_call(
            73,
            73,
            mitosis::rpc_handlers::RPCId::Echo as _,
            0xdeadbeaf as u64,
        )
        .unwrap();
    let mut caller_timeout = Timeout::new(caller, timeout_usec);
    let res = block_on(&mut caller_timeout);
    match res {
        Ok(v) => {
            let (_, reply) = v;
            log::debug!("sanity check client call result: {:?}", reply);
        }
        Err(e) => log::error!("client call error: {:?}", e),
    };

    // now we connect to the second thread, and then test
    let mut caller = caller_timeout.into_inner();
    caller
        .connect(
            73 + 1,
            73 + 1,
            client_session_1,
            UDHyperMeta {
                gid: context.query_gid(1, 0).unwrap(),
                service_id: mitosis::rdma_context::SERVICE_ID_BASE - 1,
                qd_hint: 0,
                ..Default::default()
            },
        )
        .unwrap();
    let mut caller_timeout = Timeout::new(caller, timeout_usec);
    let res = block_on(&mut caller_timeout);
    match res {
        Ok(_) => {}
        Err(e) => log::error!("client call error: {:?}", e),
    };

    // now call the second session with echo
    let mut caller = caller_timeout.into_inner();
    caller
        .sync_call(
            73 + 1,
            73 + 1,
            mitosis::rpc_handlers::RPCId::Echo as _,
            0xdeadbeaf as u64,
        )
        .unwrap();
    let mut caller_timeout = Timeout::new(caller, timeout_usec);
    let res = block_on(&mut caller_timeout);
    match res {
        Ok(v) => {
            let (_, reply) = v;
            log::debug!("sanity check client call result: {:?}", reply);
        }
        Err(e) => log::error!("client call error: {:?}", e),
    };
}

#[krdma_test(test_rpc, test_rpc_two)]
fn init() {
    log::info!("in test mitosis service startups!");

    let mut config: mitosis::Config = Default::default();
    config
        .set_num_nics_used(1)
        .set_rpc_threads(2)
        .set_max_core_cnt(1)
        .set_init_dc_targets(12);

    assert!(start_instance(config).is_some());
}

#[krdma_drop]
fn clean() {
    end_instance();
}
