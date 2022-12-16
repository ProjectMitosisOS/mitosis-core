use alloc::sync::Arc;
use alloc::vec::Vec;

use os_network::rdma::rc::*;
use crate::KRdmaKit::comm_manager::Explorer;
use os_network::Factory;
use os_network::KRdmaKit::context::Context;

use hashbrown::HashMap;

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
pub struct RCPool<'a>{
    pool: HashMap<usize, RCConn>,

    factories: Vec<&'a RCFactory>,

    contexts: Vec<&'a Arc<Context>>,
}


impl<'a> RCPool<'a> {
    pub fn new(config: &crate::Config) -> core::option::Option<Self>{
        let pool = HashMap::new();
        let mut factories = Vec::new();
        let mut contexts = Vec::new();

        for i in 0..config.max_core_cnt {
            let nic_idx = i % config.num_nics_used;

            let context = unsafe {
                crate::get_rdma_context_ref(nic_idx)
                    .expect("get rdma context failed.")
            };
            let factory = unsafe { crate::get_rc_factory_ref(nic_idx).unwrap() };

            factories.push(factory);
            contexts.push(context);
        }

        Some(Self {
            pool: pool,
            factories: factories,
            contexts: contexts,
        })
    }
}

impl<'a> RCPool<'a> {
    #[inline(always)]
    pub unsafe fn get_global_rc_conn(
        session_id: usize,
    ) -> core::option::Option<&'static mut RCConn> {
        crate::rc_pool::get_mut().get_rc_conn(session_id)
    }

    #[inline(always)]
    pub fn get_rc_conn(
        &'a mut self, 
        session_id: usize, 
    ) -> core::option::Option<&'a mut RCConn> {
        self.pool.get_mut(&session_id)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.factories.len()
    }

    pub fn create_rc_connection(
        &'a mut self,
        machine_id: usize,
        info: RCConnectInfo,
    ) -> core::option::Option<()> {
        let len = self.factories.len();
        for i in 0..len {
            let session_id = crate::startup::calculate_session_id(machine_id, i, len);
            if(self.pool.contains_key(&session_id)){
                crate::log::warn!("The session {} has already connected.", session_id);
                return None;
            }
            
            let rc_factory = self
                .factories
                .get(i)
                .expect("Failed to get rc factory");
            let gid = Explorer::string_to_gid(&info.gid).expect("Failed to convert string to ib_gid");
            let conn_meta = os_network::rdma::ConnMeta {
                gid: gid,
                service_id: info.service_id,
                port: 1, // default_nic_port
            };
            match rc_factory.create(conn_meta) {
                Ok(rcconn) => {
                    self.pool.insert(session_id, rcconn);
                }
                Err(_e) =>{
                    crate::log::error!("Failed to create rc connection to gid {}", &info.gid);
                    return None;
                }
            };
        }
        crate::log::info!("Create rc connection success");
        Some(())
    }
}
