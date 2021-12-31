#![no_std]
#![feature(get_mut_unchecked)]

//! Manage network connections 
//! 
//! 
//! 

extern crate alloc;

pub fn get_version() -> u64 { 
    0
}

pub mod conn;
pub mod memory_device;
pub mod rpc_device;

use KRdmaKit::SAClient;
use KRdmaKit::rust_kernel_rdma_base::ib_sa_client;

static mut SA_CLIENT: Option<SAClient> = None;

pub fn init_sa_client() {
    unsafe { SA_CLIENT = Some(SAClient::create()) };
}

pub fn deinit_sa_client() {
    unsafe { SA_CLIENT.as_ref().unwrap().reset() };
}

pub(crate) fn get_inner_sa_client() -> *mut ib_sa_client {
    unsafe {SA_CLIENT.as_ref().unwrap().get_inner_sa_client() }
}
