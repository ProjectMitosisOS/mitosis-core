#![no_std]

extern crate alloc;

use core::fmt::Write;

use rust_kernel_linux_util as log;
use rust_kernel_rdma_base::*;

use KRdmaKit::comm_manager::CMServer;
use KRdmaKit::services::UnreliableDatagramAddressService;
use KRdmaKit::KDriver;

use os_network::bytes::ToBytes;
use os_network::conn::{Factory, MetaFactory};
use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::rpc::*;
use os_network::timeout::Timeout;
use os_network::block_on;

use krdma_test::*;

const DEFAULT_PORT: u8 = 1;
const GRH_SIZE: usize = 40;
const DEFAULT_QD_HINT: usize = 73;
const DEFAULT_SERVICE_ID: u64 = 37;
const MAX_SEND_MSG: usize = 64;
const MAX_RECV_MSG: usize = 1024; // receive msg should be no smaller than MAX_SEND_MSG + 40

#[derive(thiserror_no_std::Error, Debug)]
pub enum TestError {
    #[error("Test error {0}")]
    Error(&'static str),
}

fn test_ud_session() -> Result<(), TestError> {
    let driver = unsafe { KDriver::create().unwrap() };

    // server side
    // create a CMServer to handle requests
    let server_ctx = driver
        .devices()
        .get(0)
        .ok_or(TestError::Error("Not valid device"))?
        .open_context()
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;
    let service_id = DEFAULT_SERVICE_ID;
    let ud_server = UnreliableDatagramAddressService::create();
    let _server_cm =
        CMServer::new(service_id, &ud_server, server_ctx.get_dev_ref()).map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;

    // create server-side ud
    let server_port = DEFAULT_PORT;
    let factory = UDFactory::new(&server_ctx);
    let server_ud = factory
        .create(UDCreationMeta { port: server_port })
        .map_err(|_| {
            log::error!("Create ud qp error.");
            TestError::Error("UD qp error.")
        })?;

    // register UD qp at server side
    let qd_hint = DEFAULT_QD_HINT;
    ud_server.reg_qp(qd_hint, &server_ud.get_qp());

    // client side
    let client_ctx = driver
        .devices()
        .get(1)
        .ok_or(TestError::Error("Not valid device"))?
        .open_context()
        .map_err(|_| {
            log::error!("Open client ctx error.");
            TestError::Error("Client context error.")
        })?;

    // create client side UDHyperMeta and get endpoint information
    let client_port = DEFAULT_PORT;
    let gid = server_ctx.get_dev_ref().query_gid(server_port, 0).unwrap();
    let meta = UDHyperMeta {
        gid,
        service_id,
        qd_hint,
        local_port: client_port,
    };
    let client_factory = UDFactory::new(&client_ctx);
    let endpoint = client_factory.create_meta(meta).map_err(|_| {
        log::error!("Create endpoint error.");
        TestError::Error("Endpoint error.")
    })?;

    // create client side ud qp and
    let client_ud = client_factory
        .create(UDCreationMeta { port: client_port })
        .map_err(|_| {
            log::error!("Create ud qp error.");
            TestError::Error("UD qp error.")
        })?;
    let mut client_session = client_ud.create(endpoint).unwrap();

    // post the recv buf at server side
    let mut ud_receiver = UDReceiverFactory::new()
        .set_qd_hint(qd_hint)
        .create(server_ud);
    for _ in 0..12 {
        ud_receiver
            .post_recv_buf(UDMsg::new(MAX_RECV_MSG, 0, server_ctx.clone()))
            .map_err(|_| {
                log::error!("Post recv buf error.");
                TestError::Error("Server ud post recv buf error.")
            })?;
    }

    // create a message and send at client side
    let mut request = UDMsg::new(MAX_SEND_MSG, 0, client_ctx.clone());
    write!(&mut request, "hello world").unwrap();
    client_session
        .post(&request, MAX_SEND_MSG, true)
        .map_err(|_| {
            log::error!("Post send buf error.");
            TestError::Error("Client session post send buf error.")
        })?;

    // check the message has been sent
    let timeout_usec = 5000_000;
    let mut client_session = Timeout::new(client_session, timeout_usec);
    block_on(&mut client_session).map_err(|_| {
        log::error!("Poll client session cq error.");
        TestError::Error("Client session cq poll error.")
    })?;

    // now receive the request
    let mut ud_receiver = Timeout::new(ud_receiver, timeout_usec);
    let result = block_on(&mut ud_receiver).map_err(|_| {
        log::error!("Poll server cq error.");
        TestError::Error("Server cq poll error.")
    })?;

    // check the content
    unsafe {
        let result = result.get_bytes().truncate_header(GRH_SIZE).unwrap();
        if result.clone_and_resize(request.get_bytes().len()).unwrap()
            != request.get_bytes().clone()
        {
            Err(TestError::Error("Error content."))
        } else {
            Ok(())
        }
    }
}

fn test_wrapper() -> Result<(), TestError> {
    test_ud_session()?;
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
