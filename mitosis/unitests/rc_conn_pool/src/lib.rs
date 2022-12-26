#![no_std]

extern crate alloc;

use mitosis::linux_kernel_module;
use mitosis::log;

use mitosis::startup::*;
use mitosis::rc_conn_pool::RCConnectInfo;

use mitosis::os_network;

use os_network::KRdmaKit::comm_manager::Explorer;

use krdma_test::*;

#[krdma_main]
fn kmain() {
    log::info!("in test rc_conn_pool!");

    let mut config: mitosis::Config = Default::default();  
    config
        .set_num_nics_used(1)
        .set_max_core_cnt(12);

    assert!(start_instance(config.clone()).is_some());

    unsafe { assert!(mitosis::get_rc_factory_ref(0).is_some()) }; 
    unsafe { assert!(mitosis::get_rc_factory_ref(1).is_none()) }; 

    let gid =  unsafe { mitosis::get_rc_factory_ref(0).unwrap().get_context().query_gid(1, 0).expect("failed query gid") };
    let gid = Explorer::gid_to_string(&gid);

    let info = RCConnectInfo::create(&gid, 0);

    for i in 0..config.max_core_cnt {
        let rc_pool = unsafe { mitosis::get_rc_conn_pool_mut(i).expect("failed get rc conn pool") };
        rc_pool.create_rc_connection(i, config.machine_id, info.clone());
    }

    unsafe { assert!(mitosis::get_rc_conn_pool_ref(0).is_some()) };
    unsafe { assert!(mitosis::get_rc_conn_pool_ref(12).is_none()) };
    unsafe { assert!(mitosis::get_rc_conn_pool_ref(0).unwrap().get_rc_conn(0).is_some()) }; 
    unsafe { assert!(mitosis::get_rc_conn_pool_ref(0).unwrap().get_rc_conn(1).is_none()) };
}

#[krdma_drop]
fn clean() {
    end_instance();
}