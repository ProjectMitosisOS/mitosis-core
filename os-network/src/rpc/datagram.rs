use KRdmaKit::cm::ServerCM;

use super::*;

/// The datagram caller is a caller implemented in RDMA's unreliable datagram (UD).
/// It assumes that the size of the message is within the
/// datagram packet.
/// In RDMA's UD case, maximum supported message is MTU
pub struct Caller {}

/// The hook will receive datagram requests, and call the RPC callback correspondingly.
pub struct RPCHook<'a> {
    service: Service<'a>,
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
