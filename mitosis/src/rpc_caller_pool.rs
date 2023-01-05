use alloc::sync::Arc;
use alloc::vec::Vec;

use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::rpc::impls::ud::UDSession;
use os_network::rpc::*;
use os_network::ud::UDFactory;
use os_network::Factory;
use os_network::KRdmaKit::context::Context;

#[allow(unused_imports)]
use crate::linux_kernel_module;
use crate::lock_bundler::BoxedLockBundler;
use crate::lock_bundler::LockBundler;

pub(crate) type UDCaller = Caller<UDReceiver, UDSession>;

/// The pool maintains a thread_local_pool of callers
/// Each CPU core can use the dedicated pool
#[derive(Default)]
pub struct CallerPool<'a> {
    // the major caller that is used by the clients
    pool: Vec<BoxedLockBundler<UDCaller>>,

    // each entry stored the caller to create the corresponding caller
    factories: Vec<&'a UDFactory>,

    contexts: Vec<&'a Arc<Context>>,

    // Vec<(qd_hint, service_ida)>
    metas: Vec<(usize, u64)>,
}

use os_network::MetaFactory;

impl<'a> CallerPool<'a> {
    #[inline(always)]
    pub unsafe fn get_global_caller(
        idx: usize,
    ) -> core::option::Option<&'static mut BoxedLockBundler<UDCaller>> {
        crate::service_caller_pool::get_mut().get_caller(idx)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    #[inline(always)]
    pub fn get_caller(&'a mut self, idx: usize) -> core::option::Option<&'a mut BoxedLockBundler<UDCaller>> {
        self.pool.get_mut(idx)
    }

    #[inline(always)]
    pub fn get_caller_context(&'a self, idx: usize) -> core::option::Option<&'a Arc<Context>> {
        self.contexts.get(idx).map(|r| *r)
    }

    // FIXME: currently we don't consider timeout
    pub fn connect_session_at(
        &'a mut self,
        idx: usize,
        session_id: usize,
        my_session_id : usize,
        meta: UDHyperMeta,
    ) -> core::option::Option<()> {
        // fetch by sidr connect
        let meta = self.create_meta_at(idx, meta)?;
        let my_gid = self.contexts.get(idx).unwrap().query_gid(1, 0).unwrap();
        let (hint, service_id) = self.metas.get(idx).unwrap().clone();

        let caller = self.get_caller(idx)?;
        caller.lock(|caller| {
            if caller.session_connected(session_id) {
                crate::log::warn!("The session {} has already connected.", session_id);
                return None;
            }
    
            let client_session = caller.get_transport_mut().create(meta).unwrap();
            let local_port_num = client_session.get_inner().get_qp().port_num();
    
            // send the connect message
            caller
                .connect(
                    session_id,
                    my_session_id,
                    client_session,
                    UDHyperMeta {
                        gid: my_gid,
                        service_id: service_id as _,
                        qd_hint: hint as _,
                        local_port: local_port_num,
                    },
                )
                .unwrap();
    
            // wait for the completion
            let (msg, _reply) = os_network::block_on(caller).expect("should succeed");
    
            // TODO: check the_reply is correct
    
            caller.register_recv_buf(msg).unwrap();

            Some(())
        })
    }

    #[inline]
    pub fn create_meta_at(
        &self,
        idx: usize,
        meta: UDHyperMeta,
    ) -> core::option::Option<crate::KRdmaKit::queue_pairs::DatagramEndpoint> {
        let factory = self
            .factories
            .get(idx)
            .expect("The idx is out of pool's range.");
        if let Ok(meta) = factory.create_meta(meta) {
            Some(meta)
        } else {
            None
        }
    }
}

impl<'a> CallerPool<'a> {
    pub fn new(config: &crate::Config) -> core::option::Option<Self> {
        let mut pool = Vec::new();
        let mut factories = Vec::new();
        let mut contexts = Vec::new();
        let mut metas = Vec::new();

        for i in 0..config.max_core_cnt {
            let nic_idx = i % config.num_nics_used;
            let client_ud_hint = crate::rpc_service::QD_HINT_BASE + config.rpc_threads_num * 2 + i;

            // acquire all the contexts
            let context = unsafe {
                crate::get_rdma_context_ref(nic_idx)
                    .expect("should initialize the pool before RDMA context is started.")
            };
            let factory = unsafe { crate::get_ud_factory_ref(nic_idx).unwrap() };
            let ud_service = unsafe { crate::get_ud_service_ref(nic_idx).unwrap() };
            let cm_server = unsafe { crate::get_rdma_cm_server_ref(nic_idx).unwrap() };

            // init
            let client_ud = factory.create(
                UDCreationMeta { port: config.default_nic_port, }
            ).expect("failed to create RPC UD");
            ud_service.reg_qp(client_ud_hint, &client_ud.get_qp());

            let client_receiver = UDReceiverFactory::new()
                .set_qd_hint(client_ud_hint)
                .create(client_ud.clone());

            let mut caller = UDCaller::new(client_receiver);
            for _ in 0..64 {
                caller
                    .register_recv_buf(UDMsg::new(4096, 0, client_ud.get_qp().ctx().clone()))
                    .expect("failed to register receive buffer for the RPC caller");
                // should succeed
            }
            let caller = LockBundler::new(caller);

            pool.push(caller);
            factories.push(factory);
            contexts.push(context);
            metas.push((client_ud_hint, cm_server.listen_id()))
        }

        Some(Self {
            pool: pool,
            factories: factories,
            contexts: contexts,
            metas: metas,
        })
    }
}
