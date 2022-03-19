#![no_std]

extern crate alloc;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use rust_kernel_linux_util as log;

use os_network::bytes::*;
use os_network::serialize::*;

use krdma_test::*;

use alloc::vec;

/// A test on `BytesMut`
fn test_bytes() {
    use core::fmt::Write;

    let max_buf_len = 32; 

    let mut buf = vec![0; max_buf_len];

    // this is dangerous!! just for the test
    let mut bytes = unsafe { BytesMut::from_raw(buf.as_mut_ptr(), buf.len())}; 

    write!(&mut bytes, "hello world").unwrap();

    log::info!("{:?}", bytes);

    let mut buf2 = vec![0; max_buf_len]; 
    let mut bytes_2 = unsafe { BytesMut::from_raw(buf2.as_mut_ptr(), buf2.len())}; 

    if bytes != bytes_2 { 
        log::info!("not equal before the copy"); 
    }
    assert_ne!(bytes, bytes_2); 

    assert_eq!(bytes_2.copy(&bytes, 0),true);
    
    log::info!("check copied bytes: {:?}", bytes_2);
    
    if bytes == bytes_2 { 
        log::info!("equal after copy"); 
    }
    assert_eq!(bytes, bytes_2);
}

fn test_serialize() { 
    log::info!("test serialize!"); 
    let mut buf = vec![0; 64]; 
    let test_val : u64 = 73; 

    let mut bytes = unsafe { BytesMut::from_raw(buf.as_mut_ptr(), buf.len())}; 
    unsafe { bytes.memcpy_serialize(&test_val) };
}

#[krdma_test(test_bytes,test_serialize)]
fn init() {}
