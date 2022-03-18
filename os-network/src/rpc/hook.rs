use KRdmaKit::cm::ServerCM;
use KRdmaKit::rpc::data::ReqType;

use super::*;
use crate::conn::*;
use crate::datagram::Receiver;

use crate::future::*;

use core::marker::PhantomData;

/// The hook will receive datagram requests, and call the RPC callback correspondingly.
/// - Factory: the connection factory
/// - M: the message type
/// - G: a generator to generate the request according to the generator
pub struct RPCHook<'a, F, G, M>
where
    F: 'a + Factory,
    G: super::MsgToReq<
        ReqType = <<F as Factory>::ConnType<'a> as Conn>::ReqPayload,
        MsgType = M,
    >,
{
    service: Service<'a>,
    conn_factory: F,
    _data: PhantomData<(G, M)>,
}

impl<'a, F, G, M> RPCHook<'a, F, G, M>
where
    F: 'a + Factory,
    G: super::MsgToReq<
        ReqType = <<F as Factory>::ConnType<'a> as Conn>::ReqPayload,
        MsgType = M,
    >,
{
    pub fn get_mut_service(&mut self) -> &mut Service<'a> {
        &mut self.service
    }
}

impl<'a, F, G, M> RPCHook<'a, F, G, M>
where
    F: 'a + Factory,
    G: super::MsgToReq<
        ReqType = <<F as Factory>::ConnType<'a> as Conn>::ReqPayload,
        MsgType = M,
    >,
{
    pub fn new(factory: F) -> Self {
        Self {
            service: Service::new(),
            conn_factory: factory,
            _data: PhantomData,
        }
    }
}

use crate::bytes::BytesMut;

impl<'a, F, G, M> Future for RPCHook<'a, F, G, M>
where
    F: 'a + Factory,
    G: super::MsgToReq<
        ReqType = <<F as Factory>::ConnType<'a> as Conn>::ReqPayload,
        MsgType = M,
    >,
{
    type Output = &'a [u8];
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        unimplemented!();
    }
}
