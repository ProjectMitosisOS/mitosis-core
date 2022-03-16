#![no_std]

extern crate alloc;

use core::fmt::Write;

use KRdmaKit::cm::SidrCM;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::mem::{Memory, RMemPhy};
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::KDriver;

use rust_kernel_linux_util as log;

use os_network::block_on;
use os_network::bytes::BytesMut;
use os_network::timeout::Timeout;
use os_network::rdma::ud::UDFactory;
use os_network::bytes::RMemRegion;
use os_network::Datagram;

use mitosis_macros::declare_global;

use krdma_test::*;

const ENTRY_COUNT: usize = 2048;
const ENTRY_SIZE: usize = 512;
const DEFAULT_QD_HINT: u64 = 74;
const HEAD_SIZE: usize = 40;
const MSG_SIZE: usize = ENTRY_SIZE - HEAD_SIZE;

declare_global!(KDRIVER, alloc::boxed::Box<KRdmaKit::KDriver>);

/// A test on `UDFactory`
/// Pre-requisition: `ctx_init`
fn test_ud_factory() {
    let driver = unsafe { KDRIVER::get_ref() };
    let nic = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available");
    let factory = UDFactory::new(nic);
    if factory.is_none() {
        log::error!("fail to create ud factory");
        return;
    }
    let factory = factory.unwrap();
    let ud = factory.create::<ENTRY_COUNT, ENTRY_SIZE>();
    if ud.is_err() {
        log::error!("fail to create ud qp");
        return;
    }
}

/// A test on one-sided operation on `UDDatagram`
/// Pre-requisition: `ctx_init`
fn test_ud_two_sided() {
    let timeout_usec = 5000000;
    let driver = unsafe { KDRIVER::get_ref() };
    let nic = driver
        .devices()
        .into_iter()
        .next()
        .unwrap();
    let factory = UDFactory::new(nic).unwrap();
    let ctx = factory.get_context();
    let server_ud = factory.create::<ENTRY_COUNT, ENTRY_SIZE>().unwrap();
    let mut client_ud = factory.create::<0, 0>().unwrap();

    
    let service_id: u64 = 0;
    let ctrl = RCtrl::create(service_id, &ctx).unwrap();
    ctrl.reg_ud(DEFAULT_QD_HINT as usize, server_ud.get_qp());
    let gid = ctx.get_gid_as_string();

    let path_res = factory
        .get_context()
        .explore_path(gid, service_id)
        .unwrap();
    let mut sidr_cm = SidrCM::new(ctx, core::ptr::null_mut()).unwrap();
    let endpoint = sidr_cm
        .sidr_connect(path_res, service_id, DEFAULT_QD_HINT)
        .unwrap();
    
    let mut send_mem = RMemPhy::new(MSG_SIZE);
    let mut send_bytes = unsafe {
        BytesMut::from_raw(send_mem.get_ptr() as *mut u8, send_mem.get_sz() as usize)
    };
    write!(&mut send_bytes, "hello world").unwrap();
    let mem_region = unsafe {
        RMemRegion::new(send_bytes,
                    send_mem.get_pa(0),
                    ctx.get_lkey())
    };
    let result = client_ud.post_msg(&endpoint, &mem_region);
    if result.is_err() {
        log::error!("fail to post message");
        return;
    }
    let mut timeout = Timeout::new(server_ud, timeout_usec);
    let result = block_on(&mut timeout);
    if result.is_err() {
        log::error!("polling ud qp with error");
        return;
    }

    let result = result.unwrap();
    let recv_bytes = unsafe {
        BytesMut::from_raw((result.get_bytes().get_raw() + HEAD_SIZE as u64) as *mut u8, MSG_SIZE)
    };
    let send_bytes = unsafe {
        BytesMut::from_raw(send_mem.get_ptr() as *mut u8, MSG_SIZE)
    };
    if send_bytes == recv_bytes {
        log::info!("equal after ud operation!");
    } else {
        log::error!("not equal after ud operation!");
    }

    // Remember to post the buffer back
    let mut server_ud = timeout.into_inner();
    let result = server_ud.post_recv_buf(result);
    if result.is_err() {
        log::error!("fail to post recv buffer");
    }
}

#[krdma_test(test_ud_factory, test_ud_two_sided)]
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
