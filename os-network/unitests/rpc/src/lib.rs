#![no_std]

extern crate alloc;

use alloc::vec;
use core::fmt::Write;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use rust_kernel_linux_util as log;

use os_network::bytes::*;
use os_network::rpc::*; 
use krdma_test::*;

fn test_callback(input : &mut BytesMut, output : &mut BytesMut) { 
    log::info!("test callback input {:?}", input); 
    log::info!("test callback output {:?}", output); 
}

// a local test 
fn test_service() {
    let mut service = Service::new(); 
    assert_eq!(true, service.register(73, test_callback));  
    log::info!("rpc service created! {}", service); 

    let mut buf = vec![0; 64];
    let mut msg = unsafe { BytesMut::from_raw(buf.as_mut_ptr(), buf.len())};
    write!(&mut msg, "hello world").unwrap();

    log::info!("test msg {:?}",msg);

    let mut out_buf = vec![0;64]; 
    let mut out_msg = unsafe { BytesMut::from_raw(out_buf.as_mut_ptr(), out_buf.len())}; 
    write!(&mut msg, "This is the output").unwrap(); 

    assert_eq!(true, service.execute(73, &mut msg, &mut out_msg));
}

// a test with RDMA
fn test_rpc() { 
//    let mut rpc = hook::RPCHook::new(); 
//    rpc.get_mut_service().register(73, test_callback);     
}

#[krdma_test(test_service,test_rpc)]
fn init() { }