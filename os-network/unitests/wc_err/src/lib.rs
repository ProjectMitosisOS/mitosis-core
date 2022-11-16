#![no_std]

extern crate alloc;

use alloc::sync::Arc;

use os_network::block_on;
use os_network::rdma::payload::rc::RCReqPayload;
use os_network::conn::Factory;
use os_network::rdma::payload::{RDMAOp, RDMAWR};
use os_network::{rdma, Conn};
use os_network::KRdmaKit;

use KRdmaKit::rdma_shim::bindings::*;
use KRdmaKit::rdma_shim::{linux_kernel_module, log};
use KRdmaKit::services::ReliableConnectionServer;
use KRdmaKit::comm_manager::CMServer;
use KRdmaKit::MemoryRegion;
use KRdmaKit::KDriver;

use krdma_test::*;

#[derive(thiserror_no_std::Error, Debug)]
pub enum TestError {
    #[error("Test error {0}")]
    Error(&'static str),
}

/// A test on error work completion status code of RC qp
/// 1. An error rkey will result in IB_WC_REM_ACCESS_ERR and cause the qp to be in an error state
/// 2. Further requests will cause IB_WC_WR_FLUSH_ERR
fn test_rc_post_poll_err() -> Result<(), TestError> {
    let driver = unsafe { KDriver::create().unwrap() };
    // server side
    let server_ctx = driver
        .devices()
        .get(0)
        .ok_or(TestError::Error("Not valid device"))?
        .open_context()
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;

    let server_port: u8 = 1;
    log::info!(
        "Server context's device name {}",
        server_ctx.get_dev_ref().name()
    );
    let server_service_id = 73;
    let rc_server = ReliableConnectionServer::create(&server_ctx, server_port);
    let server_cm = CMServer::new(server_service_id, &rc_server, server_ctx.get_dev_ref())
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;
    let addr = unsafe { server_cm.inner().raw_ptr() }.as_ptr() as u64;
    log::info!("Server cm addr 0x{:X}", addr);

    // client side
    let client_port: u8 = 1;
    let client_ctx = driver
        .devices()
        .get(1)
        .ok_or(TestError::Error("Not valid device"))?
        .open_context()
        .map_err(|_| {
            log::error!("Open client ctx error.");
            TestError::Error("Client context error.")
        })?;
    log::info!(
        "Client context's device name {}",
        client_ctx.get_dev_ref().name()
    );

    let conn_meta = rdma::ConnMeta {
        gid: server_ctx.get_dev_ref().query_gid(server_port, 0).unwrap(),
        port: client_port,
        service_id: server_service_id,
    };

    let client_factory = rdma::rc::RCFactory::new(&client_ctx);
    let mut rc = client_factory.create(conn_meta).map_err(|_| {
        log::error!("Error creating rc qp.");
        TestError::Error("Create rc qp error.")
    })?;
    
    // memory region
    let server_mr = Arc::new(MemoryRegion::new(server_ctx.clone(), 256).map_err(|_| {
        log::error!("Failed to create server MR.");
        TestError::Error("Create mr error.")
    })?);
    let client_mr = Arc::new(MemoryRegion::new(client_ctx.clone(), 256).map_err(|_| {
        log::error!("Failed to create client MR.");
        TestError::Error("Create mr error.")
    })?);

    let server_slot = unsafe { (server_mr.get_virt_addr() as *mut u64).as_mut().unwrap() };
    let client_slot = unsafe { (client_mr.get_virt_addr() as *mut u64).as_mut().unwrap() };
    *client_slot = 0;
    *server_slot = 87654321;

    let raddr = unsafe { server_mr.get_rdma_addr() };
    let rkey = server_mr.rkey().0;

    // post a signaled read request with an error rkey
    // this will generate a wc with IB_WC_REM_ACCESS_ERR
    let payload = RCReqPayload::new(
        client_mr.clone(),
        0..8,
        true,
        RDMAOp::READ,
        rkey+1, // pass an error rkey to the payload
        raddr,
    );
    rc.post(&payload).map_err(|_| {
        log::error!("Failed to post rc operation");
        TestError::Error("RC post request error.")
    })?;

    // poll the completion queue
    let wc = block_on(&mut rc).map_err(|_| {
        log::error!("Failed to poll rc qp");
        TestError::Error("RC poll error.")
    })?;

    if wc.status != ib_wc_status::IB_WC_REM_ACCESS_ERR {
        log::error!("Error rkey should generate IB_WC_REM_ACCESS_ERR error.");
        return Err(TestError::Error("WC status error."));
    }

    // now the qp is in an error state
    // any firther requests will result in a wc with IB_WC_WR_FLUSH_ERR
    let payload = payload
        .set_rkey(rkey);
    rc.post(&payload).map_err(|_| {
        log::error!("Failed to post read operation");
        TestError::Error("RC read error.")
    })?;

    // poll the completion queue
    let wc = block_on(&mut rc).map_err(|_| {
        log::error!("Failed to poll rc qp");
        TestError::Error("RC poll error.")
    })?;

    if wc.status != ib_wc_status::IB_WC_WR_FLUSH_ERR {
        log::error!("Error rkey should generate IB_WC_WR_FLUSH_ERR error.");
        return Err(TestError::Error("WC status error."));
    }

    Ok(())
}

fn test_wrapper() -> Result<(), TestError> {
    test_rc_post_poll_err()?;
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
