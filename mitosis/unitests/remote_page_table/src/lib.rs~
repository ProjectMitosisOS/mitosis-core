#![no_std]

extern crate alloc;

use mitosis::linux_kernel_module;
use mitosis::log;
use mitosis::rust_kernel_linux_util::kthread;

use mitosis::startup::*;
use mitosis::{os_network, KRdmaKit};

use KRdmaKit::ctrl::RCtrl;

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
            UDHyperMeta {
                // the remote machine's RDMA gid. Since we are unit test, we use the local gid
                gid: os_network::rdma::RawGID::new(context.get_gid_as_string()).unwrap(),

                // CM server on RNIC 0 listens on mitosis::rdma_context::SERVICE_ID_BASE,
                // CM server on RNIC 1 listens on mitosis::rdma_context::SERVICE_ID_BASE + 1, etc
                service_id: mitosis::rdma_context::SERVICE_ID_BASE,

                // Thread 0's UD registers on mitosis::rpc_service::QD_HINT_BASE,
                // Thread 1's UD registers on mitosis::rpc_service::QD_HINT_BASE + 1, etcc
                qd_hint: (mitosis::rpc_service::QD_HINT_BASE) as _,
            },
        )
        .expect("failed to connect the endpoint");

    // now we can call!
    let caller = unsafe {
        mitosis::rpc_caller_pool::CallerPool::get_global_caller(pool_idx)
            .expect("the caller should be properly inited")
    };

    crate::log::info!("now start to test RPC caller");

    caller
        .sync_call(
            0xdeadbeaf,                              // remote session ID
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
}

fn test_rpc() {
    type UDCaller<'a> = Caller<UDReceiver<'a>, UDSession<'a>>;

    log::info!("in test rpc");

    // we need to sleep 1 second,
    // to ensure that the RPC servers has successfully created, since they are created in an async way

    kthread::sleep(1);

    let factory = unsafe { mitosis::get_ud_factory_ref(0).expect("no available factory") };
    let context = unsafe { mitosis::get_rdma_context_ref(0).expect("no available context") };

    // init caller
    let client_ud = factory.create(()).unwrap();

    // use a disjoint service ID different than the server
    let ctrl = RCtrl::create(mitosis::rdma_context::SERVICE_ID_BASE - 1, context).unwrap();
    ctrl.reg_ud(0 as usize, client_ud.get_qp());

    let (endpoint, key) = factory
        .create_meta(UDHyperMeta {
            gid: os_network::rdma::RawGID::new(context.get_gid_as_string()).unwrap(),
            service_id: mitosis::rdma_context::SERVICE_ID_BASE,
            qd_hint: mitosis::rpc_service::QD_HINT_BASE as _,
        })
        .unwrap();

    let (endpoint1, key1) = factory
        .create_meta(UDHyperMeta {
            gid: os_network::rdma::RawGID::new(context.get_gid_as_string()).unwrap(),
            service_id: mitosis::rdma_context::SERVICE_ID_BASE,
            qd_hint: (mitosis::rpc_service::QD_HINT_BASE + 1) as _,
        })
        .unwrap();

    let lkey = unsafe { context.get_lkey() };
    let client_session = client_ud.create((endpoint, key)).unwrap();
    let client_session_1 = client_ud.create((endpoint1, key1)).unwrap();

    let client_receiver = UDReceiverFactory::new()
        .set_qd_hint(0 as _)
        .set_lkey(lkey)
        .create(client_ud);

    let mut caller = UDCaller::new(client_receiver);
    for _ in 0..12 {
        caller.register_recv_buf(UDMsg::new(4096, 73)).unwrap(); // should succeed
    }

    caller
        .connect(
            73,
            client_session,
            UDHyperMeta {
                gid: os_network::rdma::RawGID::new(context.get_gid_as_string()).unwrap(),
                service_id: mitosis::rdma_context::SERVICE_ID_BASE - 1,
                qd_hint: 0,
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
        .sync_call(73, mitosis::rpc_handlers::RPCId::Nil as _, 128 as u64)
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
            client_session_1,
            UDHyperMeta {
                gid: os_network::rdma::RawGID::new(context.get_gid_as_string()).unwrap(),
                service_id: mitosis::rdma_context::SERVICE_ID_BASE - 1,
                qd_hint: 0,
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
