#![no_std]

extern crate alloc;

use alloc::boxed::Box;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::KDriver;

use rust_kernel_linux_util as log; 

use os_network::{rdma,ConnFactory}; 

use krdma_test::*;

static mut KDRIVER: Option<Box<KDriver>> = None;

unsafe fn global_kdriver() -> &'static mut Box<KDriver> {
    match KDRIVER {
        Some(ref mut x) => &mut *x,
        None => panic!()
    }
}

fn test_rc_factory() {
    let driver = unsafe { global_kdriver() };
    let client_nic = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available"); 
    
    let mut client_factory = rdma::rc::RCFactory::new(client_nic).unwrap(); 

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

fn test_rc_factory_with_meta() {
    let driver = unsafe { global_kdriver() };
    let client_nic = driver
        .devices()
        .into_iter()
        .next()
        .expect("no rdma device available"); 
    
    let mut client_factory = rdma::rc::RCFactoryWPath::new(client_nic).unwrap(); 

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

#[krdma_test(test_rc_factory, test_rc_factory_with_meta)]
fn ctx_init() {
    unsafe {
        KDRIVER = KDriver::create();
    }
}
