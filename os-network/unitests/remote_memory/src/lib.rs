#![no_std]

extern crate alloc;

use alloc::vec;
use core::fmt::Write;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

use rust_kernel_linux_util as log;

use os_network::bytes::*;
use os_network::remote_memory::Device;

struct SampleTestModule;

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
    let mut dev = LocalDevice; 

    log::info!("pre check dst {:?}", dst);     
    dev.read(&(), &src.get_raw(), &(), &mut dst).unwrap(); 
    log::info!("after check dst {:?}", dst); 
    assert_eq!(src,dst); 
}

impl linux_kernel_module::KernelModule for SampleTestModule {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        log::info!("test started");
        test_local();
        Ok(Self {})
    }
}

linux_kernel_module::kernel_module!(
    SampleTestModule,
    author: b"xmm",
    description: b"The unit tests for testing remote memory abstractions.",
    license: b"GPL"
);
