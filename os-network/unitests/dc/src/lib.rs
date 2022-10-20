#![no_std]

extern crate alloc;
use alloc::sync::Arc;

use os_network::rdma::dc::DCFactory;
use os_network::rdma::DCCreationMeta;
use os_network::rdma::payload::RDMAOp;
use os_network::rdma::payload::dc::DCReqPayload;
use os_network::ud::{UDHyperMeta, UDFactory};
use os_network::KRdmaKit::rdma_shim::bindings::*;
use os_network::KRdmaKit::rdma_shim::{linux_kernel_module, log};
use os_network::KRdmaKit;
use os_network::block_on;
use os_network::MetaFactory;
use os_network::conn::Factory;
use os_network::timeout::Timeout;
use os_network::Conn;
use os_network::rdma::payload::{RDMAWR, LocalMR, Signaled};

use KRdmaKit::{KDriver, MemoryRegion};
use KRdmaKit::comm_manager::CMServer;
use KRdmaKit::services::dc::DCTargetService;

use krdma_test::*;

static DEFAULT_MEM_SIZE: usize = 1024;
static DEFAULT_PORT: u8 = 1;
static DEFAULT_QD_HINT: usize = 73;
static DEFAULT_DC_KEY: u64 = 37;

/// The error type of data plane operations
#[derive(thiserror_no_std::Error, Debug)]
pub enum TestError {
    #[error("Test error {0}")]
    Error(&'static str),
}

/// A test on `DCFactory`
fn test_dc_factory() -> Result<(), TestError> {
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
    let factory = DCFactory::new(&ctx);

    // Create a DC Target
    let dct_target = factory
        .create_target(DEFAULT_DC_KEY, DEFAULT_PORT)
        .ok_or_else(|| {
            log::error!("Create DC Target error.");
            TestError::Error("DC Target error.")
        })?;

    log::info!(
        "check dct key & num: [{} {}]",
        dct_target.dc_key(),
        dct_target.dct_num()
    );

    // Create a normal DC qp
    let _ = factory
        .create(DCCreationMeta { port: DEFAULT_PORT })
        .map_err(|_| {
            log::error!("Create DC qp error.");
            TestError::Error("DC qp error.")
        })?;
    Ok(())
}

/// A test on one-sided operation on `DCConn`
fn test_dc_one_sided() -> Result<(), TestError> {
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

    // start the listening service
    let server_service_id = 73;
    let server = DCTargetService::create();
    let _server_cm = CMServer::new(server_service_id, &server, server_ctx.get_dev_ref()).unwrap();

    // create and register a DC Target
    let factory = DCFactory::new(&server_ctx);
    let dct_target = factory
        .create_target(DEFAULT_DC_KEY, DEFAULT_PORT)
        .ok_or_else(|| {
            log::error!("Create DC Target error.");
            TestError::Error("DC Target error.")
        })?;
    server.reg_qp(DEFAULT_QD_HINT, &dct_target);

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
    
    // create dc qp at client side
    let client_factory = DCFactory::new(&client_ctx);
    let mut dc = client_factory
        .create(DCCreationMeta { port: DEFAULT_PORT })
        .map_err(|_| {
            log::error!("Create DC qp error.");
            TestError::Error("DC qp error.")
        })?;

    // get endpoint for DC Operation at client side
    let gid = server_ctx.get_dev_ref().query_gid(DEFAULT_PORT, 0).unwrap();
    let meta = UDHyperMeta {
        gid,
        service_id: server_service_id,
        qd_hint: DEFAULT_QD_HINT,
        local_port: DEFAULT_PORT,
    };
    let querier = UDFactory::new(&client_ctx);
    let endpoint = Arc::new(querier.create_meta(meta).map_err(|_| {
        log::error!("Create endpoint error.");
        TestError::Error("Endpoint error.")
    })?);

    // memory region
    let server_mr = Arc::new(MemoryRegion::new(server_ctx.clone(), DEFAULT_MEM_SIZE).map_err(|_| {
        log::error!("Failed to create server MR.");
        TestError::Error("Create mr error.")
    })?);
    let client_mr = Arc::new(MemoryRegion::new(client_ctx.clone(), DEFAULT_MEM_SIZE).map_err(|_| {
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
    let payload = DCReqPayload::new(
        client_mr.clone(),
        0..8,
        false,
        RDMAOp::READ,
        rkey,
        raddr,
        endpoint.clone(),
    );
    dc.post(&payload).map_err(|_| {
        log::error!("Failed to post read operation");
        TestError::Error("DC read error.")
    })?;

    // post a signaled write request
    // write 8 bytes to the raddr+8 from the local memory region indexed by 8..16
    let payload = payload
        .set_op(RDMAOp::WRITE)
        .set_raddr(raddr+8)
        .set_local_mr_range(8..16)
        .set_signaled();
    dc.post(&payload).map_err(|_| {
        log::error!("Failed to post write operation");
        TestError::Error("DC write error.")
    })?;

    // poll the completion queue
    let timeout_usec = 5000_000;
    let mut timeout = Timeout::new(dc, timeout_usec);
    block_on(&mut timeout).map_err(|_| {
        log::error!("Failed to poll dc qp");
        TestError::Error("DC poll error.")
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
    test_dc_factory()?;
    test_dc_one_sided()?;
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
