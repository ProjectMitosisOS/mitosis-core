#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::sync::Arc;

use core::fmt::Write;

use rust_kernel_linux_util::linux_kernel_module;
use rust_kernel_linux_util as log;

use KRdmaKit::KDriver;
use KRdmaKit::cm::SidrCM;
use KRdmaKit::ctrl::RCtrl;
use KRdmaKit::mem::{RMemPhy, Memory};

use os_network::Factory;
use os_network::bytes::*;
use os_network::rdma::dc::DCFactory;
use os_network::remote_memory::Device;
use os_network::remote_memory::rdma::{DCRemoteDevice, DCKey, LocalMemoryBuffer};

use krdma_test::*;

static MEM_SIZE: usize = 1024;
static DEFAULT_QD_HINT: u64 = 73;

/// A test on `LocalDevice`
fn test_local() {

    // init context
    let max_buf_len = 32; 

    let mut buf0 = vec![0; max_buf_len];

    // this is dangerous!! just for the test
    let mut src = unsafe { BytesMut::from_raw(buf0.as_mut_ptr(), buf0.len())}; 

    write!(&mut src, "hello world").unwrap();

    let mut buf1 = vec![0; max_buf_len];
    let mut dst = unsafe { BytesMut::from_raw(buf1.as_mut_ptr(), buf1.len())}; 
    assert_ne!(src,dst); 

    use os_network::remote_memory::local::LocalDevice;    
    let mut dev = LocalDevice::<(),(), os_network::rdma::Err>::new(); 

    log::info!("pre check dst {:?}", dst);     
    dev.read(&(), &src.get_raw(), &(), &mut dst).unwrap(); 
    log::info!("after check dst {:?}", dst); 
    assert_eq!(src,dst); 
}


/// A test on `DCRemoteDevice`
fn test_dc_remote() {
    let driver = unsafe {
        KDriver::create().unwrap()
    };

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
    let client_factory = DCFactory::new(client_nic).unwrap();
    let client_ctx = client_factory.get_context();
    let lkey = unsafe { client_ctx.get_lkey() };
    let dc = client_factory.create(()).unwrap();
    let dc = Arc::new(dc);

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
    let mut mem = RMemPhy::new(MEM_SIZE);
    let local_bytes = unsafe { BytesMut::from_raw(mem.get_ptr() as _, MEM_SIZE/2 as usize) };
    let mut remote_bytes = unsafe { BytesMut::from_raw(mem.get_ptr().add(MEM_SIZE/2) as _, MEM_SIZE/2 as usize) };

    write!(&mut remote_bytes, "hello world").unwrap();

    let mut dev = DCRemoteDevice::new(dc);
    let res = dev.read(&endpoint,
                    &mem.get_pa((MEM_SIZE/2) as u64),
                    &DCKey::new(lkey, rkey, 73),
                    &mut LocalMemoryBuffer::new(mem.get_pa(0), MEM_SIZE/2));
    if res.is_err() {
        log::error!("unable to read remote memory");
        return;
    }

    if local_bytes == remote_bytes {
        log::info!("equal after remote memory read operation!");
    } else {
        log::error!("not equal after remote memory read operation!");
        log::info!("local {:?}", local_bytes); 
        log::info!("remote {:?}", remote_bytes); 
    }
}

#[krdma_test(test_local, test_dc_remote)]
fn ctx_init() {
    // do nothing
}
