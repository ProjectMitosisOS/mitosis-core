use KRdmaKit::cm::ServerCM;

use crate::datagram::Receiver;
use super::*;
use crate::conn::*;

/// The hook will receive datagram requests, and call the RPC callback correspondingly.
pub struct RPCHook<F : Factory, 'a> {
    service: Service<'a>,
    factory: F
}

impl<'a> RPCHook<'a> {
    pub fn get_mut_service(&mut self) -> &mut Service<'a> {
        &mut self.service
    }
}

impl<'a> RPCHook<'a> {
    pub fn new() -> Self {
        Self {
            service: Service::new(),
        }
    }
}
