#![no_std]

extern crate alloc;

mod console_msgs;

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module::{c_types};
use KRdmaKit::rust_kernel_rdma_base::*;
use KRdmaKit::thread_local::ThreadLocal;
use KRdmaKit::device::{RNIC, RContext};
use linux_kernel_module::println;

use alloc::vec::Vec;

use lazy_static::lazy_static;

type UnsafeGlobal<T> = ThreadLocal<T>;

use network_daemon::conn::RDMAConn;
use network_daemon::conn::rc::{RCService, RCConn};

struct RCConnTestModule {
}

lazy_static! {
    static ref ALLNICS: UnsafeGlobal<Vec<RNIC>> = UnsafeGlobal::new(Vec::new());
    static ref ALLRCONTEXTS: UnsafeGlobal<Vec<RContext<'static>>> = UnsafeGlobal::new(Vec::new());
}

unsafe extern "C" fn _add_one(dev: *mut ib_device) {
    let nic = RNIC::create(dev, 1);
    ALLNICS.get_mut().push(nic.ok().unwrap());
}

gen_add_dev_func!(_add_one, _new_add_one);

unsafe extern "C" fn _remove_one(dev: *mut ib_device, _client_data: *mut c_types::c_void) {
    println!("remove one dev {:?}", dev);
}

static mut CLIENT: Option<ib_client> = None;

unsafe fn get_global_client() -> &'static mut ib_client {
    match CLIENT {
        Some(ref mut x) => &mut *x,
        None => panic!(),
    }
}

fn print_test_msgs(test_case_idx: usize, assert: bool) {
    if assert {
        println!("{:?}", crate::console_msgs::SUCC[test_case_idx]);
    } else {
        println!("{:?}", crate::console_msgs::ERR[test_case_idx]);
    }
}

fn ctx_init() {
    // register
    unsafe {
        CLIENT = Some(core::mem::MaybeUninit::zeroed().assume_init());
        get_global_client().name = b"kRdmaKit-unit-test\0".as_ptr() as *mut c_types::c_char;
        get_global_client().add = Some(_new_add_one);
        get_global_client().remove = Some(_remove_one);
        get_global_client().get_net_dev_by_params = None;
    }

    let err = unsafe { ib_register_client(get_global_client() as *mut ib_client) };
    print_test_msgs(0, err == 0);
    print_test_msgs(0, ALLNICS.len() > 0);

    // create all of the context according to NIC
    for i in 0..ALLNICS.len() {
        ALLRCONTEXTS.get_mut()
            .push(RContext::create(&mut ALLNICS.get_mut()[i]).unwrap());
        println!("create [{}] success", i);
    }

    // sa_client
    network_daemon::init_sa_client();
}

impl linux_kernel_module::KernelModule for RCConnTestModule {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        ctx_init();
        test_connection();
        Ok(Self {})
    }
}

fn test_connection() {
    let target_gid = ALLRCONTEXTS.get_mut()[0].get_gid_as_string();
    let service_id = 50 as usize;
    let remote_service_id = service_id as u64;
    let test_ctx = &ALLRCONTEXTS.get_mut()[0];
    let qd_hint = 666;
    let rc_service = RCService::new(service_id, test_ctx);
    if rc_service.is_none() {
        println!("BUG: RCService failed to initialize");
        return
    }
    let rc_conn = RCConn::new(&target_gid, test_ctx, remote_service_id, qd_hint);
    if rc_conn.is_none() {
        println!("BUG: RCConn failed to initialize");
        return;
    }
    let mut rc_conn = rc_conn.unwrap();
    let res = rc_conn.connect(&target_gid);
    if res.is_err() {
        println!("BUG: connect failed for RCConn");
        return;
    }
    println!("test_connection passed!");
}

impl Drop for RCConnTestModule {
    fn drop(&mut self) {
        unsafe { ib_unregister_client(get_global_client() as *mut ib_client) };
        network_daemon::deinit_sa_client();
        ALLRCONTEXTS.get_mut().clear();
    }
}

linux_kernel_module::kernel_module!(
    RCConnTestModule,
    author: b"xmm",
    description: b"RC Connection Test",
    license: b"GPL"
);
