use core::panic;

#[allow(unused_imports)]
use crate::linux_kernel_module;

use crate::{
    bytes::{BytesMut, ToBytes},
    Receiver,
};
use hashbrown::HashMap;

pub enum Err {
    /// Timeout error
    Timeout = 0,
    NoID = 1,
}

pub trait GetTransport {
    type Transport;

    fn get_transport_mut(&mut self) -> &mut Self::Transport;
}

pub struct Caller<R: Receiver, S: RPCConn> {
    inner_receiver: R,
    connected_sessions: HashMap<usize, (S, S::ReqPayload)>,
}

impl<R, S: RPCConn> GetTransport for Caller<R, S>
where
    R: Receiver + GetTransport,
{
    type Transport = R::Transport;
    fn get_transport_mut(&mut self) -> &mut Self::Transport {
        self.inner_receiver.get_transport_mut()
    }
}

impl<R, SS> Caller<R, SS>
where
    R: Receiver,
    SS: RPCConn,
    SS::ReqPayload: ToBytes,
{
    pub fn get_pending_reqs(&self, session_id: usize) -> core::option::Option<usize> {
        self.connected_sessions
            .get(&session_id)
            .map(|(s, _)| s.get_pending_reqs())
    }

    /// Note, this call should use Future to wait
    pub fn sync_call<Args>(
        &mut self,
        session_id: usize,
        my_session_id : usize,
        rpc_id: usize,
        arg: Args,
    ) -> Result<(), SS::IOResult> {
        let (session, msg) = self
            .connected_sessions
            .get_mut(&session_id)
            .expect("failed to get session");

        let signal_flag = session.get_pending_reqs() == 0;
        let req_sz = header_factory::CallStubFactory::new(my_session_id, rpc_id)
            .generate(&arg, msg.get_bytes_mut())
            .unwrap();
        #[cfg(feature = "resume-profile")]
        crate::log::info!("rpc req size:{}", req_sz);
        session.post(msg, req_sz, signal_flag)?;

        // 8 is really really a magic number
        if session.get_pending_reqs() > 8 {    
            let res = crate::block_on(session); // should never fail
            assert!(res.is_ok());
            assert!(session.get_pending_reqs() == 0);
        }        

        Ok(())
    }

    pub fn session_connected(&self, session_id: usize) -> bool {
        // crate::log::info!("check: {:?}", self.connected_sessions.keys());
        self.connected_sessions.contains_key(&session_id)
    }

    /// Note before the connect, one should register_recv_buf for receiving the ack
    ///
    /// # Arguments
    /// * session_id: the user-defined session IF
    /// * s : the initialized transport that is ready to send to the server
    /// * meta : the meta of s, so that the remote end can communicate with it
    ///
    /// # Note
    /// This call will crash if the session has already connected
    pub fn connect(
        &mut self,
        session_id: usize,
        my_session_id : usize,
        mut s: SS,
        meta: SS::HyperMeta,
    ) -> Result<(), SS::IOResult>
    where
        SS::ReqPayload: ToBytes,
    {
        let mut msg_buf = SS::ReqPayload::create(R::MTU, 0);
        let req_sz = ConnectStubFactory::new(my_session_id)
            .generate(&meta, msg_buf.get_bytes_mut())
            .unwrap();
        s.post(&msg_buf, req_sz, true)?;

        // FIXME: timeout?
        // In our UD-based session, it will never happen
        // Yet, we should handle this
        match crate::block_on(&mut s) {
            Ok(_) => {}
            _ => panic!(),
        };

        self.connected_sessions.insert(session_id, (s, msg_buf));
        Ok(())
    }

    pub fn get_ss(&self, session_id: usize) -> Option<&(SS, <SS as RPCConn<SS>>::ReqPayload)> {
        self.connected_sessions.get(&session_id)
    }
}

impl<R, SS> Caller<R, SS>
where
    R: Receiver,
    SS: RPCConn,
{
    pub fn new(inner: R) -> Self {
        Self {
            inner_receiver: inner,
            connected_sessions: HashMap::new(),
        }
    }

    pub fn register_recv_buf(&mut self, msg: R::MsgBuf) -> Result<(), R::IOResult> {
        self.inner_receiver.post_recv_buf(msg)
    }
}

use crate::future::*;

impl<R, SS> crate::future::Future for Caller<R, SS>
where
    R: Receiver,
    SS: RPCConn,
    R::Output: ToBytes,
{
    type Output = (R::Output, BytesMut);
    // registered message, the reply
    type Error = CallError<R::Error>;

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        match self.inner_receiver.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(msg)) => {
                let mut msg_header: header::MsgHeader = Default::default();

                let bytes = unsafe { msg.get_bytes().clone() };
                let msg_header_bytes = unsafe { bytes.truncate_header(R::HEADER).unwrap() };
                unsafe { msg_header_bytes.memcpy_deserialize(&mut msg_header) };

                if msg_header.is_reply() {
                    match msg_header.get_reply_stub() {
                        Some(stub) => {
                            match stub {
                                header::ReplyStatus::Ok => {
                                    // parse and return the detailed message
                                    Ok(Async::Ready((msg, unsafe {
                                        msg_header_bytes
                                            .truncate_header(
                                                core::mem::size_of::<header::MsgHeader>(),
                                            )
                                            .unwrap()
                                            .clone_and_resize(msg_header.get_payload())
                                            .unwrap()
                                    })))
                                }
                                header::ReplyStatus::NotExist => Err(CallError::no_id()),
                            }
                        }
                        None => Err(CallError::fatal()),
                    }
                } else {
                    Err(CallError::fatal())
                }
            }
            Err(e) => Err(CallError::inner(e)),
        }
    }
}

pub mod analysis;
pub mod hook;

pub mod header;
pub mod header_factory;

pub use header_factory::*;

// modules for registering RPC callbacks
pub mod service;

pub use service::Service;

/// This is a simple wrapper over crate::conn::Conn
/// The reason for doing so is to simplify customization for RPC
pub trait RPCConn<T: Future = Self>: Future {
    type ReqPayload: AllocMsgBuf;
    // the request format
    type CompPayload = Self::Output;
    type IOResult = Self::Error;
    type HyperMeta;

    // post the request to the underlying device
    fn post(
        &mut self,
        req: &Self::ReqPayload,
        sz: usize,
        signaled: bool,
    ) -> Result<(), Self::IOResult>;

    // a call specific to RDMA
    fn get_pending_reqs(&self) -> usize;
}

/// This is a simple wrapper over crate::conn::Factory
/// The reason for doing so is to simplify customization for RPC
pub trait RPCFactory {
    type ConnMeta;
    type ConnType: RPCConn;
    type ConnResult;

    // create and connect the connection
    fn create<'a>(&'a self, meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult>;
}

pub trait AllocMsgBuf {
    fn create(size: usize, imm: u32) -> Self;
}

/// The connection should provide a GenHyperMeta trait,
/// such that the RPC hook can use it to create a session corresponding to the sender
pub trait GenHyperMeta<F: crate::conn::MetaFactory> {
    type Args;

    fn generate_hyper(&self, args: &Self::Args) -> F::HyperMeta;
}

// concrete implementations based on real transports
pub mod impls;

// RPC call error
#[derive(Debug)]
pub struct CallError<T>(CallKind<T>);

/// Timeout error variants
#[derive(Debug)]
enum CallKind<T> {
    /// Inner value returned an error
    Inner(T),

    /// No ID associated with the call function
    NoID,

    ConnectError,

    Fatal,
}

impl<T> CallError<T> {
    /// Create a new `Error` representing the inner value completing with `Err`.
    pub fn inner(err: T) -> CallError<T> {
        CallError(CallKind::Inner(err))
    }

    /// Returns `true` if the error was caused by the inner value completing
    /// with `Err`.
    pub fn is_inner(&self) -> bool {
        match self.0 {
            CallKind::Inner(_) => true,
            _ => false,
        }
    }

    /// Create a new `Error` representing the function ID is not registered
    pub fn no_id() -> CallError<T> {
        CallError(CallKind::NoID)
    }

    pub fn fatal() -> CallError<T> {
        CallError(CallKind::Fatal)
    }

    pub fn connect_error() -> CallError<T> {
        CallError(CallKind::ConnectError)
    }
}
