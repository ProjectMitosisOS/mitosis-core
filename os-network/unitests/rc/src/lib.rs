#![no_std]

extern crate alloc;

use alloc::sync::Arc;

use KRdmaKit::services::ReliableConnectionServer;
use KRdmaKit::comm_manager::CMServer;
use KRdmaKit::MemoryRegion;

use rust_kernel_rdma_base::linux_kernel_module;
use rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use rust_kernel_linux_util as log;

use os_network::block_on;
use os_network::rdma::payload::rc::RCReqPayload;
use os_network::conn::Factory;
use os_network::timeout::Timeout;
use os_network::rdma::payload::{RDMAOp, RDMAWR, LocalMR, Signaled};
use os_network::{rdma, Conn};

use krdma_test::*;

/// The error type of data plane operations
#[derive(thiserror_no_std::Error, Debug)]
pub enum TestError {
    #[error("Test error {0}")]
    Error(&'static str),
}

/// A test on `RCFactory`
fn test_rc_factory() -> Result<(), TestError> {
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

    let client_factory = rdma::rc::RCFactory::new(client_ctx.clone());
    let client_qp = client_factory.create(conn_meta).map_err(|_| {
        log::error!("Error creating rc qp.");
        TestError::Error("Create rc qp error.")
    })?;
    let status = client_qp.get_status().map_err(|_| {
        log::error!("Error getting qp status.");
        TestError::Error("Get qp status error.")
    })?;
    
    if status != KRdmaKit::queue_pairs::QueuePairStatus::ReadyToSend {
        log::error!("Error qp status: {:?}", status);
        Err(TestError::Error("Error qp status."))
    } else {
        Ok(())
    }
}

fn test_rc_post_poll() -> Result<(), TestError> {
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

    let client_factory = rdma::rc::RCFactory::new(client_ctx.clone());
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

    let server_slot_0 = unsafe { (server_mr.get_virt_addr() as *mut u64).as_mut().unwrap() };
    let client_slot_0 = unsafe { (client_mr.get_virt_addr() as *mut u64).as_mut().unwrap() };
    let server_slot_1 = unsafe {
        ((server_mr.get_virt_addr() + 8) as *mut u64)
            .as_mut()
            .unwrap()
    };
    let client_slot_1 = unsafe {
        ((client_mr.get_virt_addr() + 8) as *mut u64)
            .as_mut()
            .unwrap()
    };
    *client_slot_0 = 0;
    *client_slot_1 = 12345678;
    *server_slot_0 = 87654321;
    *server_slot_1 = 0;
    log::info!("[Before] client: v0 {} v1 {}", client_slot_0, client_slot_1);
    log::info!("[Before] server: v0 {} v1 {}", server_slot_0, server_slot_1);

    let raddr = unsafe { server_mr.get_rdma_addr() };
    let rkey = server_mr.rkey().0;

    // post a unsignaled read request
    // read 8 bytes from raddr to the local memory region indexed by 0..8
    let payload = RCReqPayload::new(
        client_mr.clone(),
        0..8,
        false,
        RDMAOp::READ,
        rkey,
        raddr,
    );
    rc.post(&payload).map_err(|_| {
        log::error!("Failed to post read operation");
        TestError::Error("RC read error.")
    })?;

    // post a signaled write request
    // write 8 bytes to the raddr+8 from the local memory region indexed by 8..16
    let payload = payload
        .set_op(RDMAOp::WRITE)
        .set_raddr(raddr+8)
        .set_local_mr_range(8..16)
        .set_signaled();
    rc.post(&payload).map_err(|_| {
        log::error!("Failed to post write operation");
        TestError::Error("RC write error.")
    })?;

    // poll the completion queue
    let timeout_usec = 5000_000;
    let mut timeout = Timeout::new(rc, timeout_usec);
    block_on(&mut timeout).map_err(|_| {
        log::error!("Failed to poll rc qp");
        TestError::Error("RC poll error.")
    })?;

    log::info!("[After ] client: v0 {} v1 {}", client_slot_0, client_slot_1);
    log::info!("[After ] server: v0 {} v1 {}", server_slot_0, server_slot_1);

    // check the results
    if *client_slot_0 == *server_slot_0 && *client_slot_1 == *server_slot_1 {
        Ok(())
    } else {
        Err(TestError::Error("read write failed"))
    }
}

fn test_wrapper() -> Result<(), TestError> {
    test_rc_factory()?;
    test_rc_post_poll()?;
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
