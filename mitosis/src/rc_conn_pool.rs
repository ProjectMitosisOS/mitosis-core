use alloc::sync::Arc;
use alloc::vec::Vec;

use os_network::rdma::rc::*;
use crate::KRdmaKit::comm_manager::Explorer;
use os_network::Factory;
use os_network::KRdmaKit::context::Context;
use os_network::rdma::rc::RCConn;

use hashbrown::HashMap;

use crate::linux_kernel_module::bindings::mutex;

pub struct RCConnectInfo {
    pub gid: alloc::string::String,
    pub service_id: u64,
    pub local_port: u8,
}

impl RCConnectInfo {
    #[inline]
    pub fn create(gid: &alloc::string::String, nic_idx: u64) -> Self {
        Self::create_with_port(gid, nic_idx, 1)  // default port number
    }

    #[inline]
    pub fn create_with_port(gid: &alloc::string::String, nic_idx: u64, port_num: u8) -> Self {
        let service_id = crate::rdma_context::RC_SERVICE_ID_BASE as u64
                        + nic_idx % unsafe { (*crate::max_nics_used::get_ref()) as u64 };
        Self {
            gid: gid.clone(),
            service_id,
            local_port: port_num,
        }
    }
}

impl Clone for RCConnectInfo {
    fn clone(&self) -> RCConnectInfo {
        Self {
            gid: self.gid.clone(),
            service_id: self.service_id,
            local_port: self.local_port,
        }
    }
}

#[derive(Default)]
pub struct RCPool{
    pool: HashMap<usize, RCConn>,

    mutex: mutex,
}

impl RCPool {
    pub fn new() -> core::option::Option<Self>{
        let pool = HashMap::new();

        let m: mutex = Default::default();
        m.init();

        Some(Self {
            pool: pool,
            mutex: m,
        })
    }
}

impl<'a> RCPool {
    #[inline(always)]
    pub fn get_rc_conn(
        &'a self, 
        session_id: usize, 
    ) -> core::option::Option<&RCConn> {
        self.mutex.lock();
        let ret = self.pool.get(&session_id);
        self.mutex.unlock();
        ret
    }

    pub fn create_rc_connection(
        &'a mut self,
        idx: usize,
        machine_id: usize,
        info: RCConnectInfo,
    ) -> core::option::Option<()> {
        let session_id = unsafe {crate::startup::calculate_session_id(
            machine_id, 
            idx, 
            *crate::max_caller_num::get_ref()
        ) };
        let i = idx % unsafe{ *crate::max_nics_used::get_ref() };

        let rc_factory = unsafe { crate::get_rc_factory_ref(i) }.expect("fatal, should not fail to get rc factory");
        
        let gid = Explorer::string_to_gid(&info.gid).expect("Failed to convert string to ib_gid");
        let conn_meta = os_network::rdma::ConnMeta {
            gid: gid,
            service_id: info.service_id,
            port: 1, // default_nic_port
        };

        self.mutex.lock();
        if self.pool.contains_key(&session_id) {
            crate::log::warn!("The session {} has already connected.", session_id);
            self.mutex.unlock();
            return None;
        } 

        match rc_factory.create(conn_meta) {
            Ok(rcconn) => {
                self.pool.insert(session_id, rcconn);
                self.mutex.unlock();
            }
            Err(_e) =>{
                crate::log::error!("Failed to create rc connection to gid {}", &info.gid);
                self.mutex.unlock();
                return None;
            }
        };
        Some(())
    }
}
