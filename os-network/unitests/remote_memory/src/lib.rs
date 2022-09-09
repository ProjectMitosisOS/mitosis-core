#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use alloc::vec;

use core::fmt::Write;

use os_network::block_on;
use os_network::bytes::*;
use os_network::rdma::dc::DCFactory;
use os_network::rdma::rc::RCFactory;
use os_network::rdma::ConnMeta;
use os_network::rdma::DCCreationMeta;
use os_network::remote_memory::rdma::{DCKeys, DCRemoteDevice};
use os_network::remote_memory::rdma::{RCKeys, RCRemoteDevice};
use os_network::remote_memory::Device;
use os_network::timeout::TimeoutWRef;
use os_network::ud::UDFactory;
use os_network::ud::UDHyperMeta;
use os_network::Factory;
use os_network::MetaFactory;
use os_network::KRdmaKit;
use os_network::KRdmaKit::rdma_shim::{linux_kernel_module, log};

use KRdmaKit::comm_manager::CMServer;
use KRdmaKit::services::dc::DCTargetService;
use KRdmaKit::services::ReliableConnectionServer;
use KRdmaKit::KDriver;
use KRdmaKit::MemoryRegion;

use krdma_test::*;

static DEFAULT_MEM_SIZE: usize = 1024;
static DEFAULT_QD_HINT: usize = 73;
static DEFAULT_PORT: u8 = 1;
static DEFAULT_DC_KEY: u64 = 37;

#[derive(thiserror_no_std::Error, Debug)]
pub enum TestError {
    #[error("Test error {0}")]
    Error(&'static str),
}

/// A test on `LocalDevice`
fn test_local() -> Result<(), TestError> {
    // init context
    let max_buf_len = 32;

    let mut buf0 = vec![0; max_buf_len];

    // this is dangerous!! just for the test
    let mut src = unsafe { BytesMut::from_raw(buf0.as_mut_ptr(), buf0.len()) };

    write!(&mut src, "hello world").unwrap();

    let mut buf1 = vec![0; max_buf_len];
    let mut dst = unsafe { BytesMut::from_raw(buf1.as_mut_ptr(), buf1.len()) };
    assert_ne!(src, dst);

    use os_network::remote_memory::local::LocalDevice;
    let mut dev = LocalDevice::<(), (), os_network::rdma::Err>::new();

    unsafe { dev.read(&(), &src.get_raw(), &(), &mut dst, &()).unwrap() };
    if src == dst {
        Ok(())
    } else {
        log::error!("No equal after local read");
        log::error!("source: {:?}", src);
        log::error!("destination: {:?}", dst);
        Err(TestError::Error("Local read error."))
    }
}

/// A test on `DCRemoteDevice`
fn test_dc_remote() -> Result<(), TestError> {
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
    let dc = client_factory
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
    let server_mr = Arc::new(
        MemoryRegion::new(server_ctx.clone(), DEFAULT_MEM_SIZE).map_err(|_| {
            log::error!("Failed to create server MR.");
            TestError::Error("Create mr error.")
        })?,
    );
    let client_mr = Arc::new(
        MemoryRegion::new(client_ctx.clone(), DEFAULT_MEM_SIZE).map_err(|_| {
            log::error!("Failed to create client MR.");
            TestError::Error("Create mr error.")
        })?,
    );

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

    let mut laddr = unsafe { client_mr.get_rdma_addr() };
    let raddr = unsafe { server_mr.get_rdma_addr() };
    let rkey = server_mr.rkey().0;

    // read 8 bytes from remote raddr to the local laddr
    let mut remote_device = DCRemoteDevice::new(dc);
    unsafe {
        remote_device.read(
            endpoint.as_ref(),
            &raddr,
            &DCKeys::new(rkey),
            &mut laddr,
            &8,
        )
    }
    .map_err(|_| {
        log::error!("Failed to issue read request to DCRemoteDevice.");
        TestError::Error("DCRemoteDevice read error.")
    })?;

    // poll the completion for the read request
    let timeout_usec = 5000_000;
    let mut timeout = TimeoutWRef::new(&mut remote_device, timeout_usec);
    block_on(&mut timeout).map_err(|_| {
        log::error!("Failed to poll result from DCRemoteDevice.");
        TestError::Error("DCRemoteDevice poll error.")
    })?;

    // write 8 bytes to the remote (raddr+8) to local (laddr+8)
    unsafe {
        remote_device.write(
            endpoint.as_ref(),
            &(raddr + 8),
            &DCKeys::new(rkey),
            &mut (laddr + 8),
            &8,
        )
    }
    .map_err(|_| {
        log::error!("Failed to issue read request to DCRemoteDevice.");
        TestError::Error("DCRemoteDevice read error.")
    })?;

    // poll the completion for the write request
    let mut timeout = TimeoutWRef::new(&mut remote_device, timeout_usec);
    block_on(&mut timeout).map_err(|_| {
        log::error!("Failed to poll result from DCRemoteDevice.");
        TestError::Error("DCRemoteDevice poll error.")
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

/// A test on `RCRemoteDevice`
fn test_rc_remote() -> Result<(), TestError> {
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

    let server_port: u8 = DEFAULT_PORT;
    let server_service_id = 73;
    let rc_server = ReliableConnectionServer::create(&server_ctx, server_port);
    let _server_cm = CMServer::new(server_service_id, &rc_server, server_ctx.get_dev_ref())
        .map_err(|_| {
            log::error!("Open server ctx error.");
            TestError::Error("Server context error.")
        })?;

    // client side
    let client_port: u8 = DEFAULT_PORT;
    let client_ctx = driver
        .devices()
        .get(1)
        .ok_or(TestError::Error("Not valid device"))?
        .open_context()
        .map_err(|_| {
            log::error!("Open client ctx error.");
            TestError::Error("Client context error.")
        })?;

    let conn_meta = ConnMeta {
        gid: server_ctx.get_dev_ref().query_gid(server_port, 0).unwrap(),
        port: client_port,
        service_id: server_service_id,
    };

    let client_factory = RCFactory::new(client_ctx.clone());
    let rc = client_factory.create(conn_meta).map_err(|_| {
        log::error!("Error creating rc qp.");
        TestError::Error("Create rc qp error.")
    })?;

    // memory region
    let server_mr = Arc::new(
        MemoryRegion::new(server_ctx.clone(), DEFAULT_MEM_SIZE).map_err(|_| {
            log::error!("Failed to create server MR.");
            TestError::Error("Create mr error.")
        })?,
    );
    let client_mr = Arc::new(
        MemoryRegion::new(client_ctx.clone(), DEFAULT_MEM_SIZE).map_err(|_| {
            log::error!("Failed to create client MR.");
            TestError::Error("Create mr error.")
        })?,
    );

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

    let mut laddr = unsafe { client_mr.get_rdma_addr() };
    let raddr = unsafe { server_mr.get_rdma_addr() };
    let rkey = server_mr.rkey().0;

    // read 8 bytes from remote raddr to the local laddr
    let mut remote_device = RCRemoteDevice::new(rc);
    unsafe { remote_device.read(&(), &raddr, &RCKeys::new(rkey), &mut laddr, &8) }.map_err(
        |_| {
            log::error!("Failed to issue read request to RCRemoteDevice.");
            TestError::Error("RCRemoteDevice read error.")
        },
    )?;

    // poll the completion for the read request
    let timeout_usec = 5000_000;
    let mut timeout = TimeoutWRef::new(&mut remote_device, timeout_usec);
    block_on(&mut timeout).map_err(|_| {
        log::error!("Failed to poll result from RCRemoteDevice.");
        TestError::Error("RCRemoteDevice poll error.")
    })?;

    // write 8 bytes to the remote (raddr+8) to local (laddr+8)
    unsafe { remote_device.write(&(), &(raddr + 8), &RCKeys::new(rkey), &mut (laddr + 8), &8) }
        .map_err(|_| {
            log::error!("Failed to issue write request to RCRemoteDevice.");
            TestError::Error("RCRemoteDevice write error.")
        })?;
    let mut timeout = TimeoutWRef::new(&mut remote_device, timeout_usec);
    block_on(&mut timeout).map_err(|_| {
        log::error!("Failed to poll result from RCRemoteDevice.");
        TestError::Error("RCRemoteDevice poll error.")
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
    test_local()?;
    test_rc_remote()?;
    test_dc_remote()?;
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
