#![no_std]

extern crate alloc;

use core::pin::Pin;

use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::mem::{Memory, RMemPhy};
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use rust_kernel_linux_util as log;

use os_network::rdma::payload;
use os_network::{rdma, Conn};
use os_network::conn::Factory;
use os_network::block_on;
use os_network::rdma::WCStatus::*;

use mitosis_macros::declare_global;

use krdma_test::*;

type RCReqPayload = payload::Payload<ib_rdma_wr>;

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);

/// A test on error work completion status code of RC qp
/// 1. An error rkey will result in IB_WC_REM_ACCESS_ERR and cause the qp to be in an error state
/// 2. Further requests will cause IB_WC_WR_FLUSH_ERR
/// Pre-requisition: `ctx_init`
fn test_rc_wc_err() {
    let driver = unsafe { KDRIVER::get_ref() };
    let client_nic = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available");

    let client_factory = rdma::rc::RCFactory::new(client_nic).unwrap();

    let server_ctx = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available")
        .open()
        .unwrap();

    let server_service_id: u64 = 0;
    let _ctrl = RCtrl::create(server_service_id, &server_ctx);

    let conn_meta = rdma::ConnMeta {
        gid: server_ctx.get_gid_as_string(),
        service_id: server_service_id,
        qd_hint: 0,
    };

    let mut rc = client_factory.create(conn_meta).unwrap();

    // prepare memory for test
    let capacity: usize = 32;
    let mut mem = RMemPhy::new(capacity);

    let mut req = RCReqPayload::default()
        .set_laddr(mem.get_pa(0))
        .set_raddr(mem.get_pa(0) + (capacity/2) as u64)
        .set_sz(capacity/2)
        .set_lkey(unsafe { client_factory.get_context().get_lkey() })
        .set_rkey(unsafe { server_ctx.get_rkey() + 1 }) // here we use a error rkey, resulting in IB_WC_REM_ACCESS_ERR, aka remote access error
        .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
        .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ);

    let mut req = unsafe { Pin::new_unchecked(&mut req) }; 
    os_network::rdma::payload::Payload::<ib_rdma_wr>::finalize(req.as_mut());

    let res = rc.post(&req.as_ref());
    if res.is_err() {
        log::error!("unable to post read op");
        return;
    }

    let result = block_on(&mut rc);
    if result.is_err() {
        let wc_status = result.err().unwrap().unwrap_wc_err();
        if wc_status != IB_WC_REM_ACCESS_ERR {
            log::error!("poll with error: {:?}, expected: {:?}", wc_status, IB_WC_REM_ACCESS_ERR);
        }
    } else {
        log::error!("should poll with error");
        return;
    }

    // Now the rc qp is in an error state, any further normal requests will result in a IB_WC_WR_FLUSH_ERR

    let mut req = RCReqPayload::default()
        .set_laddr(mem.get_pa(0))
        .set_raddr(mem.get_pa(0) + (capacity/2) as u64)
        .set_sz(capacity/2)
        .set_lkey(unsafe { client_factory.get_context().get_lkey() })
        .set_rkey(unsafe { server_ctx.get_rkey() }) 
        .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
        .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ);

    let mut req = unsafe { Pin::new_unchecked(&mut req) }; 
    os_network::rdma::payload::Payload::<ib_rdma_wr>::finalize(req.as_mut());

    let res = rc.post(&req.as_ref());
    if res.is_err() {
        log::error!("unable to post read op");
        return;
    }

    let result = block_on(&mut rc);
    if result.is_err() {
        let wc_status = result.err().unwrap().unwrap_wc_err();
        if wc_status != IB_WC_WR_FLUSH_ERR {
            log::error!("poll with error: {:?}, expected: {:?}", wc_status, IB_WC_WR_FLUSH_ERR);
        }
    } else {
        log::error!("should poll with error");
        return;
    }
}

#[krdma_test(
    test_rc_wc_err
)]
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
