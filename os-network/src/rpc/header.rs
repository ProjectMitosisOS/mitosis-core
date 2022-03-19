use KRdmaKit::qp::conn::Request;

use crate::Conn;

#[derive(Debug, Default, Copy, Clone)]
pub struct CallStub {
    session_id: usize,
    rpc_id: usize,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ConnectStub(usize);

#[derive(Debug, Default, Copy, Clone)]
#[repr(u8)]
pub enum ReplyStatus {
    #[default]
    Ok = 1, // a success call
    NotExist = 3, // function is not registered in the service
}

#[derive(Debug, Default)]
#[repr(u64)]
enum RPCMeta {
    Request(CallStub),
    Reply(ReplyStatus),
    Connect(ConnectStub),
    #[default]
    None,
}
/// Metadata of RPC messages
#[derive(Debug, Default, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ReqType {
    #[default]
    Illegal = 4,
    Connect = 0,
    Request = 1,
    Reply = 2,
    DisConnect = 3,
}

#[derive(Debug, Default)]
pub struct MsgHeader {
    marker: ReqType,
    payload: usize,
    meta: RPCMeta,
}

impl MsgHeader {
    pub fn gen_connect_stub(session_id: usize, payload: usize) -> Self {
        Self {
            marker: ReqType::Connect,
            payload: payload,
            meta: RPCMeta::Connect(ConnectStub(session_id)),
        }
    }

    pub fn gen_call_stub(session_id: usize, rpc_id: usize, payload: usize) -> Self {
        Self {
            marker: ReqType::Request,
            payload: payload,
            meta: RPCMeta::Request(CallStub {
                session_id: session_id,
                rpc_id: rpc_id,
            }),
        }
    }

    #[inline]
    pub fn get_call_stub(&self) -> core::option::Option<&CallStub> {
        match &self.meta {
            RPCMeta::Request(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn get_connect_stub(&self) -> core::option::Option<&ConnectStub> {
        match &self.meta {
            RPCMeta::Connect(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn get_marker(&self) -> ReqType {
        self.marker
    }

    pub fn is_connect(&self) -> bool {
        self.marker == ReqType::Connect
    }

    pub fn is_request(&self) -> bool {
        self.marker == ReqType::Request
    }

    pub fn is_reply(&self) -> bool {
        self.marker == ReqType::Reply
    }

    pub fn is_disconnect(&self) -> bool {
        self.marker == ReqType::DisConnect
    }
}
