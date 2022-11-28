use alloc::vec::Vec;

use os_network::{KRdmaKit::{self, services::UnreliableDatagramAddressService, comm_manager::CMServer, services::dc::DCTargetService, services::rc::ReliableConnectionServer}, rdma::dc::DCFactory};

pub const SERVICE_ID_BASE: u64 = 73; // not using 0 to prevent error
pub const GLOBAL_DC_KEY: u64 = 73;
pub const RC_SERVICE_ID_BASE: u64 = 33;

/// Note: need to call `end_rdma` before destroying the kernel module
/// Return
/// * Some(()), the start succeeds
/// * None, fatal error happens, the result is printed
pub fn start_rdma(config: &crate::Config) -> core::option::Option<()> {
    unsafe {
        let kdriver = KRdmaKit::KDriver::create()?;
        crate::rdma_driver::init(kdriver);
    };

    unsafe {
        let mut contexts = Vec::new();
        for i in 0..config.num_nics_used {
            contexts.push(
                crate::rdma_driver::get_mut()
                    .devices()
                    .get(i)
                    .expect("no available RDMA NIC")
                    .open_context()
                    .expect("failed to create RDMA context"),
            );
        }
        crate::rdma_contexts::init(contexts);
    };

    unsafe {
        let mut ud_services = Vec::new();
        for _ in 0..config.num_nics_used {
            ud_services.push(
                UnreliableDatagramAddressService::create()
            );
        }
        crate::ud_service::init(ud_services);
    };

    unsafe {
        let mut dc_targets = Vec::new();
        let mut dc_target_metas = Vec::new();
        for i in 0..config.num_nics_used {
            let factory = DCFactory::new(crate::get_rdma_context_ref(i).unwrap());
            let dct_target = factory.create_target(GLOBAL_DC_KEY, config.default_nic_port).unwrap();
            let dct_target_meta = dct_target.get_datagram_meta().expect("should not fail");
            dc_targets.push(dct_target);
            dc_target_metas.push(dct_target_meta);
        }
        crate::dc_target::init(dc_targets);
        crate::dc_target_meta::init(dc_target_metas);
    }

    unsafe {
        let mut servers = Vec::new();
        for i in 0..config.num_nics_used {
            servers.push(
                CMServer::new(
                    SERVICE_ID_BASE + i as u64,
                    crate::get_ud_service_ref(i).expect("fatal: cannot get the created ud service"),
                    crate::get_rdma_context_ref(i).expect("fatal: cannot get the created context").get_dev_ref(),
                ).expect("fail to create cm server on NIC")
            )
        }
        crate::rdma_cm_service::init(servers);
    };

    #[cfg(feature = "use_rc")]
    unsafe{
        let mut rc_servers = Vec::new();
        let mut rc_cm_servers = Vec::new();
        for i in 0..config.num_nics_used{
            let rc_server = ReliableConnectionServer::create(crate::get_rdma_context_ref(i).unwrap(), config.default_nic_port);
            let server_cm = CMServer::new(
                RC_SERVICE_ID_BASE + i as u64,
                &rc_server,
                crate::get_rdma_context_ref(i).expect("fatal: cannot get the created context").get_dev_ref(),
            ).expect("fail to create rc cm server on NIC");
            rc_servers.push(rc_server);
            rc_cm_servers.push(server_cm);
        }
        crate::rc_service::init(rc_servers);
        crate::rc_cm_service::init(rc_cm_servers);
    };

    Some(())
}

pub fn end_rdma() {
    // Note: the **order** of drop is very important here
    unsafe {
        #[cfg(feature = "use_rc")]
        crate::rc_cm_service::drop();
        #[cfg(feature = "use_rc")]
        crate::rc_service::drop();
        crate::rdma_cm_service::drop();
        crate::ud_service::drop();
        crate::dc_target_meta::drop();
        crate::dc_target::drop();
        crate::rdma_contexts::drop();
        crate::rdma_driver::drop();
    };
}
