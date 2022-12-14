/// Data structures used to generate the RPC request struct 
#[derive(Debug, Default, Copy, Clone)]
pub struct CallStub {
    session_id: usize,
    rpc_id: usize,
}

impl CallStub { 
    pub fn get_session_id(&self) -> usize {
        self.session_id
    }

    pub fn get_rpc_id(&self) -> usize { 
        self.rpc_id
    }
}

/// Data structures used to generate the RPC connect struct 
#[derive(Debug, Default, Copy, Clone)]
pub struct ConnectStub(usize);

impl ConnectStub {
    pub fn get_session_id(&self) -> usize {
        self.0
    }
}

/// Data structures used to generate the RPC reply struct 
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

    pub fn gen_reply_stub(status : ReplyStatus, sz : usize) -> Self { 
        Self { 
            marker : ReqType::Reply,
            payload : sz, 
            meta : RPCMeta::Reply(status)
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
    pub fn get_reply_stub(&self) -> core::option::Option<&ReplyStatus> {
        match &self.meta {
            RPCMeta::Reply(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn get_marker(&self) -> ReqType {
        self.marker
    }

    #[inline]
    pub fn get_payload(&self) -> usize { 
        self.payload
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
