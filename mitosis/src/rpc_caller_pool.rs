use alloc::vec::Vec;

use os_network::datagram::msg::UDMsg;
use os_network::datagram::ud::*;
use os_network::datagram::ud_receiver::*;
use os_network::rpc::impls::ud::UDSession;
use os_network::rpc::*;
use os_network::ud::UDFactory;
use os_network::Factory;
use os_network::KRdmaKit::device::RContext;

#[allow(unused_imports)]
use crate::linux_kernel_module;

type UDCaller<'a> = Caller<UDReceiver<'a>, UDSession<'a>>;

/// The pool maintains a thread_local_pool of callers
/// Each CPU core can use the dedicated pool
pub struct CallerPool<'a> {
    // the major caller that is used by the clients
    pool: Vec<UDCaller<'a>>,

    // each entry stored the caller to create the corresponding caller
    factories: Vec<&'a UDFactory<'a>>,

    contexts: Vec<&'a RContext<'a>>,

    // Vec<(qd_hint, service_ida)>
    metas: Vec<(usize, u64)>,
}

use os_network::MetaFactory;

impl<'a> CallerPool<'a> {
    #[inline(always)]
    pub unsafe fn get_global_caller(
        idx: usize,
    ) -> core::option::Option<&'static mut UDCaller<'static>> {
        crate::service_caller_pool::get_mut().get_caller(idx)
    }

    #[inline(always)]
    pub fn len(&self) -> usize { 
        self.pool.len()
    }

    #[inline(always)]
    pub fn get_caller(&'a mut self, idx: usize) -> core::option::Option<&'a mut UDCaller<'a>> {
        self.pool.get_mut(idx)
    }

    #[inline(always)]
    pub fn get_caller_context(&'a self, idx: usize) -> core::option::Option<&'a RContext<'a>> {
        self.contexts.get(idx).map(|r| *r)
    }

    // FIXME: currently we don't consider timeout
    pub fn connect_session_at(
        &'a mut self,
        idx: usize,
        session_id: usize,
        meta: UDHyperMeta,
    ) -> core::option::Option<()> {
        // fetch by sidr connect
        let meta = self.create_meta_at(idx, meta)?;
        let my_gid =
            os_network::rdma::RawGID::new(self.contexts.get(idx).unwrap().get_gid_as_string())
                .unwrap();
        let (hint, service_id) = self.metas.get(idx).unwrap().clone();

        let caller = self.get_caller(idx)?;

        let client_session = caller.get_transport_mut().create(meta).unwrap();

        // send the connect message
        caller
            .connect(
                session_id,
                client_session,
                UDHyperMeta {
                    gid: my_gid,
                    service_id: service_id as _,
                    qd_hint: hint as _,
                },
            )
            .unwrap();
            
        // wait for the completion
        let (msg, _reply) = os_network::block_on(caller).expect("should succeed");

        // TODO: check the_reply is correct

        caller.register_recv_buf(msg).unwrap();

        Some(())
    }

    #[inline]
    pub fn create_meta_at(
        &self,
        idx: usize,
        meta: UDHyperMeta,
    ) -> core::option::Option<(crate::KRdmaKit::cm::EndPoint, u32)> {
        let factory = self
            .factories
            .get(idx)
            .expect("The idx is out of pool's range.");
        if let Ok(meta) = factory.create_meta(meta) {
            Some(meta)
        }else {
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
            let cm_server = unsafe { crate::get_rdma_cm_server_ref(nic_idx).unwrap() };

            // init
            let client_ud = factory.create(()).expect("failed to create RPC UD");
            cm_server.reg_ud(client_ud_hint, client_ud.get_qp());

            let client_receiver = UDReceiverFactory::new()
                .set_qd_hint(client_ud_hint)
                .set_lkey(unsafe { context.get_lkey() })
                .create(client_ud);

            let mut caller = UDCaller::new(client_receiver);
            for _ in 0..64 {
                caller
                    .register_recv_buf(UDMsg::new(4096, 73))
                    .expect("failed to register receive buffer for the RPC caller");
                // should succeed
            }

            pool.push(caller);
            factories.push(factory);
            contexts.push(context);
            metas.push((client_ud_hint, cm_server.get_id()))
        }

        Some(Self {
            pool: pool,
            factories: factories,
            contexts: contexts,
            metas: metas,
        })
    }
}
