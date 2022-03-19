#[derive(Debug,Default)]
pub struct SessionID(usize);

#[derive(Debug,Default)]
#[repr(u8)]
pub enum ReplyStatus {
    #[default]    
    Ok = 1,       // a success call
    NotExist = 3, // function is not registered in the service
}

#[derive(Debug,Default)]
#[repr(u64)]
enum RPCMeta {
    Request(SessionID), // session ID
    Reply(ReplyStatus),
    #[default]
    None,
}
/// Metadata of RPC messages
#[derive(Debug,Default,PartialEq)]
#[repr(u8)]
enum ReqType {
    #[default]    
    Connect = 0,
    Request = 1,    
    Reply = 2,
    DisConnect = 3,
}

#[derive(Debug,Default)]
pub struct MsgHeader {
    marker: ReqType,
    payload: usize,
    meta: RPCMeta,
}

impl MsgHeader {
    pub fn new_connect_request<Meta: Sized>() -> Self {
        Self {
            marker: ReqType::Connect,
            payload: core::mem::size_of::<Meta>(),
            meta: RPCMeta::None,
        }
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
