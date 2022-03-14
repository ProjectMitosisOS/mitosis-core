#![no_std]

extern crate alloc;

use core::fmt::Write;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering::SeqCst;

use KRdmaKit::cm::SidrCM;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::mem::{Memory, RMemPhy};
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use rust_kernel_linux_util as log;

use os_network::block_on;
use os_network::bytes::BytesMut;
use os_network::rdma::payload;
use os_network::timeout::Timeout;
use os_network::{rdma, Conn, ConnFactory};

use mitosis_macros::declare_global;

use krdma_test::*;

type DCReqPayload = payload::Payload<ib_dc_wr>;

static MEM_SIZE: usize = 1024;
static DEFAULT_QD_HINT: u64 = 73;

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);

/// A test on `DCFactory`
/// Pre-requisition: `ctx_init`
fn test_dc_factory() {
    let driver = unsafe { KDRIVER::get_ref() };
    let nic = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available");

    let factory = rdma::dc::DCFactory::new(nic);
    if factory.is_none() {
        log::error!("fail to create dc factory");
        return;
    }
    let factory = factory.unwrap();

    let dc = factory.create(());
    if dc.is_err() {
        log::error!("fail to create dc qp");
        return;
    }
}

/// A test on one-sided operation on `DCConn`
/// Pre-requisition: `ctx_init`
fn test_dc_one_sided() {
    let timeout_usec = 5000000;
    let driver = unsafe { KDRIVER::get_ref() };

    // Prepare for server side RCtrl
    let server_ctx = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available")
        .open()
        .unwrap();
    let service_id: u64 = 0;
    let gid = server_ctx.get_gid_as_string();
    let rkey = unsafe { server_ctx.get_rkey() };
    let _ctrl = RCtrl::create(service_id, &server_ctx);

    // Create the dc qp
    let client_nic = driver.devices().into_iter().next().unwrap();
    let client_factory = rdma::dc::DCFactory::new(client_nic).unwrap();
    let client_ctx = client_factory.get_context();
    let lkey = unsafe { client_ctx.get_lkey() };
    let mut dc = client_factory.create(()).unwrap();

    // Prepare for the EndPoint
    let path_res = client_factory
        .get_context()
        .explore_path(gid, service_id)
        .unwrap();
    let mut sidr_cm = SidrCM::new(client_ctx, core::ptr::null_mut()).unwrap();
    let endpoint = sidr_cm
        .sidr_connect(path_res, service_id, DEFAULT_QD_HINT)
        .unwrap();

    // Prepare memory regions
    let mut local = RMemPhy::new(MEM_SIZE);
    let local_bytes = unsafe { BytesMut::from_raw(local.get_ptr() as _, local.get_sz() as usize) };
    let mut remote = RMemPhy::new(MEM_SIZE);
    let mut remote_bytes =
        unsafe { BytesMut::from_raw(remote.get_ptr() as _, remote.get_sz() as usize) };

    // Initialize the remote memory
    write!(&mut remote_bytes, "hello world from remote").unwrap();

    // Perform a remote read
    let payload = DCReqPayload::default()
        .set_laddr(local.get_pa(0))
        .set_raddr(remote.get_pa(0))
        .set_sz(MEM_SIZE)
        .set_lkey(lkey)
        .set_rkey(rkey)
        .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
        .set_opcode(ib_wr_opcode::IB_WR_RDMA_WRITE)
        .set_ah(&endpoint);
    let res = dc.post(&payload);
    if res.is_err() {
        log::error!("unable to post read qp");
        return;
    }

    let mut timeout = Timeout::new(dc, timeout_usec);
    let result = block_on(&mut timeout);
    if result.is_err() {
        log::error!("polling dc qp with error");
        return;
    }
    compiler_fence(SeqCst);

    // Memory regions should be the same after the operations
    if local_bytes == remote_bytes {
        log::info!("equal after dc read operation!");
    } else {
        log::error!(
            "not equal after dc read operation!"
        );
        log::info!("local {:?}", local_bytes); 
        log::info!("remote {:?}", remote_bytes); 
    }
}

#[krdma_test(test_dc_factory, test_dc_one_sided)]
fn ctx_init() {
    unsafe {
        KDRIVER::init(KDriver::create().unwrap());
    }
}

#[krdma_drop]
fn ctx_drop() {
    unsafe {
        KDRIVER::drop();
    }
}
