#![no_std]

extern crate alloc;
use alloc::sync::Arc;

use core::fmt::Write;

use KRdmaKit::KDriver;
use KRdmaKit::comm_manager::CMServer;
use KRdmaKit::services::UnreliableDatagramAddressService;

use rust_kernel_rdma_base::linux_kernel_module;
use rust_kernel_rdma_base::*;
use rust_kernel_linux_util as log;

use os_network::block_on;
use os_network::Receiver;
use os_network::MetaFactory;
use os_network::conn::{Conn, Factory};
use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::bytes::ToBytes;
use os_network::UDCreationMeta;
use os_network::timeout::Timeout;
use os_network::rdma::payload::ud::UDReqPayload;
use os_network::ud_receiver::UDReceiverFactory;

use krdma_test::*;

const GRH_SIZE: usize = 40;

/// The error type of data plane operations
#[derive(thiserror_no_std::Error, Debug)]
pub enum TestError {
    #[error("Test error {0}")]
    Error(&'static str),
}

/// A test on `UDFactory`
fn test_ud_factory() -> Result<(), TestError> {
    let driver = unsafe { KDriver::create().unwrap() };
    let ctx = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available")
        .open_context()
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;
    let factory = UDFactory::new(&ctx);
    let port = 1;
    let _ud = factory.create(UDCreationMeta {port})
            .map_err(|_| {
                log::error!("Create ud qp error.");
                TestError::Error("UD qp error.")
            })?;
    Ok(())
}

/// A test on one-sided operation on `UDDatagram`
fn test_ud_two_sided() -> Result<(), TestError> {
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
    let service_id = 73;
    let ud_server = UnreliableDatagramAddressService::create();
    let _server_cm = CMServer::new(service_id, &ud_server, server_ctx.get_dev_ref())
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;
    
    // create server-side ud
    let server_port = 1;
    let factory = UDFactory::new(&server_ctx);
    let ud = factory.create(UDCreationMeta {port: server_port})
            .map_err(|_| {
                log::error!("Create ud qp error.");
                TestError::Error("UD qp error.")
            })?;
    
    // register UD qp at server side
    let qd_hint = 37;
    ud_server.reg_qp(qd_hint, &ud.get_qp());

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
    
    // create client side UDHyperMeta
    let client_port = 1;
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

    // create client side ud qp
    let client_port = 1;
    let mut client_ud = client_factory.create(UDCreationMeta {port: client_port})
        .map_err(|_| {
            log::error!("Create ud qp error.");
            TestError::Error("UD qp error.")
        })?;
    
    // create memory region for both server and client
    let server_recv_buf_len = 512;
    let client_send_buf_len = 256;
    let server_recv_buf = UDMsg::new(server_recv_buf_len, 0, server_ctx.clone());
    let mut client_send_buf = UDMsg::new(client_send_buf_len, 0, client_ctx.clone());
    write!(&mut client_send_buf, "hello world").unwrap();

    // post the recv buf at server side
    let mut ud_receiver = UDReceiverFactory::new()
        .set_qd_hint(qd_hint)
        .create(ud);
    ud_receiver.post_recv_buf(server_recv_buf).map_err(|_| {
        log::error!("Post recv buf error.");
        TestError::Error("Server ud post recv buf error.")
    })?;
    
    // post the send buf at client side
    let payload = UDReqPayload::new(
        client_send_buf.get_inner(),
        0..client_send_buf_len as u64,
        true,
        Arc::new(endpoint)
    );
    client_ud.post(&payload).map_err(|_| {
        log::error!("Post send buf error.");
        TestError::Error("Client ud post send buf error.")
    })?;

    // poll at client side
    let timeout_usec = 1000_000;
    let mut client_ud = Timeout::new(client_ud, timeout_usec);
    block_on(&mut client_ud).map_err(|_| {
        log::error!("Poll client cq error.");
        TestError::Error("Client cq poll error.")
    })?;

    // poll at server side
    let mut ud_receiver = Timeout::new(ud_receiver, timeout_usec);
    let result = block_on(&mut ud_receiver).map_err(|_| {
        log::error!("Poll server cq error.");
        TestError::Error("Server cq poll error.")
    })?;

    // check the content
    unsafe {
        let result = result.get_bytes().truncate_header(GRH_SIZE).unwrap();
        if result.clone_and_resize(client_send_buf.get_bytes().len()).unwrap()
        != client_send_buf.get_bytes().clone() {
            Err(TestError::Error("Error content."))
        } else {
            Ok(())
        }
    }
}

fn test_wrapper() -> Result<(), TestError> {
    test_ud_factory()?;
    test_ud_two_sided()?;
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
