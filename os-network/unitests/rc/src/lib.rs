#![no_std]

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use KRdmaKit::ctrl::RCtrl;

use rust_kernel_linux_util as log; 

use os_network::{rdma,ConnFactory}; 

struct SampleTestModule;

fn test_rc_factory() {
    use KRdmaKit::KDriver;
    let driver = unsafe { KDriver::create().unwrap() };
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
    log::debug!("{:?}", rc.get_status().unwrap());  
}

fn test_rc_factory_with_meta() {}

impl linux_kernel_module::KernelModule for SampleTestModule {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        log::info!("test started"); 
        test_rc_factory(); 
        Ok(Self {})
    }
}

linux_kernel_module::kernel_module!(
    SampleTestModule,
    author: b"xmm",
    description: b"A sample module for unit testing",
    license: b"GPL"
);
