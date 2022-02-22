#![no_std]

extern crate alloc;

use core::fmt::Write;

use alloc::boxed::Box;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::KDriver;
use KRdmaKit::mem::{RMemPhy, Memory};
use KRdmaKit::rust_kernel_rdma_base::*;

use rust_kernel_linux_util as log; 

use os_network::{rdma, ConnFactory, Conn}; 
use os_network::bytes::BytesMut;
use os_network::rdma::payload::rc_payload::RCReqPayload;

use krdma_test::*;

static mut KDRIVER: Option<Box<KDriver>> = None;

unsafe fn global_kdriver() -> &'static mut Box<KDriver> {
    match KDRIVER {
        Some(ref mut x) => &mut *x,
        None => panic!()
    }
}

/// A test on `RCFactory`
/// Pre-requisition: `ctx_init`
fn test_rc_factory() {
    let driver = unsafe { global_kdriver() };
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


    // the main test body
    let conn_meta = rdma::ConnMeta { 
        gid : server_ctx.get_gid_as_string(), 
        service_id : server_service_id, 
        qd_hint : 0
    }; 

    let mut rc = client_factory.create(conn_meta).unwrap();
    let status = rc.get_status();
    if status.is_some() { 
        log::info!("test connect w/o meta passes! {:?}", status.unwrap()); 
    } else { 
        log::error!("unable to get the connection meta"); 
    }
}

/// A test on `RCFactoryWPath`
/// Pre-requisition: `ctx_init`
fn test_rc_factory_with_meta() {
    let driver = unsafe { global_kdriver() };
    let client_nic = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available"); 
    
    let client_factory = rdma::rc::RCFactoryWPath::new(client_nic).unwrap(); 

    let server_ctx = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available")
        .open()
        .unwrap();

    let server_service_id: u64 = 0;
    let _ctrl = RCtrl::create(server_service_id, &server_ctx);

    // the main test body
    let conn_meta = rdma::ConnMeta { 
        gid : server_ctx.get_gid_as_string(), 
        service_id : server_service_id, 
        qd_hint : 0
    }; 
    let conn_meta = client_factory.convert_meta(conn_meta).unwrap(); 

    let mut rc = client_factory.create(conn_meta).unwrap();
    let status = rc.get_status();
    if status.is_some() { 
        log::info!("test connect w/o meta passes! {:?}", status.unwrap()); 
    } else { 
        log::error!("unable to get the connection meta"); 
    }    
}

/// A test on post and poll operations on `RCConn`
/// Pre-requisition: `ctx_init`
fn test_rc_post_poll() {
    let driver = unsafe { global_kdriver() };
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
        gid : server_ctx.get_gid_as_string(), 
        service_id : server_service_id, 
        qd_hint : 0
    };

    let mut rc = client_factory.create(conn_meta).unwrap();
    
    // prepare 2 slices of memory for the post/poll
    let capacity: usize = 32;
    let mut local = RMemPhy::new(capacity);
    let mut local_bytes = unsafe { BytesMut::from_raw(local.get_ptr() as _, local.get_sz() as usize) };
    let mut remote = RMemPhy::new(capacity);
    let mut remote_bytes = unsafe { BytesMut::from_raw(remote.get_ptr() as _, remote.get_sz() as usize) };

    write!(&mut remote_bytes, "hello world from remote").unwrap();

    // read the remote memory to local
    let rc_read_req_payload = RCReqPayload::default()
                                .set_laddr(local.get_pa(0))
                                .set_raddr(remote.get_pa(0))
                                .set_sz(capacity)
                                .set_lkey(unsafe { client_factory.get_context().get_lkey() })
                                .set_rkey(unsafe { server_ctx.get_rkey() }) // here we are testing on a single machine
                                .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
                                .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ);

    let res = rc.post(&rc_read_req_payload);
    if res.is_err() {
        log::error!("unable to post read op");
        return
    }

    let res = rc.wait_til_comp();
    if res.is_err() {
        log::error!("unable to poll completion from RC qp");
        return
    }

    if local_bytes == remote_bytes {
        log::info!("equal after rdma read operation!")
    } else {
        log::error!("not equal after rdma read operation!")
    }

    write!(&mut local_bytes, "hello world from local").unwrap();

    // write local memory to remote
    let rc_write_req_payload = RCReqPayload::default()
                                .set_laddr(local.get_pa(0))
                                .set_raddr(remote.get_pa(0))
                                .set_sz(capacity)
                                .set_lkey(unsafe { client_factory.get_context().get_lkey() })
                                .set_rkey(unsafe { server_ctx.get_rkey() }) // here we are testing on a single machine
                                .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
                                .set_opcode(ib_wr_opcode::IB_WR_RDMA_WRITE);

    let res = rc.post(&rc_write_req_payload);
    if res.is_err() {
        log::error!("unable to post read op");
        return
    }

    let res = rc.wait_til_comp();
    if res.is_err() {
        log::error!("unable to poll completion from RC qp");
        return
    }

    if local_bytes == remote_bytes {
        log::info!("equal after rdma write operation!")
    } else {
        log::error!("not equal after rdma write operation!")
    }
}

#[krdma_test(test_rc_factory, test_rc_factory_with_meta, test_rc_post_poll)]
fn ctx_init() {
    unsafe {
        KDRIVER = KDriver::create();
    }
}
