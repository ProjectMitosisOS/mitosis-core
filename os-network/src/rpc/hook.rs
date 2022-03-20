use KRdmaKit::cm::ServerCM;
use KRdmaKit::rpc::data::ReqType;

use hashbrown::HashMap;

use super::*;
use crate::datagram::Receiver;
use crate::{bytes::ToBytes, conn::*};

use crate::future::*;

/// The hook will receive datagram requests, and call the RPC callback correspondingly.
/// * Factory: the connection factory that creates the session
/// * R: Receiver
pub struct RPCHook<'a, 'b, F, R, MF>
where
    F: 'a + RPCFactory,
    R: Receiver,
{
    service: Service<'a>,
    session_factory: F,
    meta_factory: &'b MF,
    transport: R,
    connected_sessions: HashMap<usize, F::ConnType<'a>>,
}

impl<'a, 'b, F, R, MF> RPCHook<'a, 'b, F, R, MF>
where
    F: 'a + RPCFactory,
    R: Receiver,
{
    pub fn get_mut_service(&mut self) -> &mut Service<'a> {
        &mut self.service
    }
}

impl<'a, 'b, F, R, MF> RPCHook<'a, 'b, F, R, MF>
where
    F: 'a + RPCFactory,
    R: Receiver,
{
    pub fn new(meta_f: &'b MF, factory: F, transport: R) -> Self {
        Self {
            service: Service::new(),
            meta_factory: meta_f,
            session_factory: factory,
            connected_sessions: HashMap::new(),
            transport: transport,
        }
    }

    pub fn post_msg_buf(&mut self, msg: R::MsgBuf) -> Result<(), R::IOResult> {
        self.transport.post_recv_buf(msg)
    }
}

use super::header::*;
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

impl<'a, 'b, F, R, MF> Future for RPCHook<'a, 'b, F, R, MF>
where
    F: 'a + RPCFactory,
    // we need to ensure that the polled result can be sent back to
    R: Receiver<
        Output = <<F as RPCFactory>::ConnType<'a> as RPCConn>::ReqPayload,
        MsgBuf = <<F as RPCFactory>::ConnType<'a> as RPCConn>::ReqPayload,
        IOResult = <R as Future>::Error,
    >,
    <<F as RPCFactory>::ConnType<'a> as RPCConn>::ReqPayload: ToBytes,
{
    type Output = (); // msg, session ID
    type Error = Error<R::Error>;

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        match self.transport.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(msg)) => {
                // parse the request
                let mut bytes = unsafe { msg.get_bytes().clone() };
                let mut msg_header: MsgHeader = Default::default();

                let mut msg_header_bytes =
                    unsafe { bytes.truncate_header(R::HEADER).ok_or(Error::corrupted()) }?;
                unsafe { msg_header_bytes.memcpy_deserialize(&mut msg_header) };

                let mut rpc_args = unsafe {
                    msg_header_bytes
                        .truncate_header(core::mem::size_of::<super::header::MsgHeader>())
                        .ok_or(Error::corrupted())?
                };

                match msg_header.get_marker() {
                    super::header::ReqType::Connect => {
                        let meta = msg_header.get_connect_stub().ok_or(Error::corrupted())?;
                        crate::log::info!("check meta in connect {:?}", meta);
                    }
                    super::header::ReqType::Request => {
                        let meta = msg_header.get_call_stub().ok_or(Error::corrupted())?;
                        crate::log::info!("check meta in {:?}", meta);

                        // TODO: call the message
                    }
                    _ => {
                        // should never happen at the hooker!
                        crate::log::error!("unknown message header {:?}", msg_header);
                    }
                }

                self.transport
                    .post_recv_buf(msg)
                    .map_err(|e| Error::inner(e))?;
                Ok(Async::NotReady)
            }
            Err(e) => Err(Error::inner(e)),
        }
    }
}

use core::fmt::{Debug, Display, Formatter};

impl<'a, 'b, F, R, MF> Debug for RPCHook<'a, 'b, F, R, MF>
where
    F: 'a + RPCFactory,
    R: Receiver,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "RPCHook service {}", self.service)
    }
}

impl<'a, 'b, F, R, MF> Display for RPCHook<'a, 'b, F, R, MF>
where
    F: 'a + RPCFactory,
    R: Receiver,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Error<T>(Kind<T>);

/// Timeout error variants
#[derive(Debug)]
enum Kind<T> {
    /// Inner value returned an error
    Inner(T),

    /// No ID associated with the call function
    NoID,

    /// The header is corrupted
    CorruptedHeader,
}

impl<T> Error<T> {
    /// Create a new `Error` representing the inner value completing with `Err`.
    pub fn inner(err: T) -> Error<T> {
        Error(Kind::Inner(err))
    }

    /// Returns `true` if the error was caused by the inner value completing
    /// with `Err`.
    pub fn is_inner(&self) -> bool {
        match self.0 {
            Kind::Inner(_) => true,
            _ => false,
        }
    }

    /// Create a new `Error` representing the RPC meta data is corrupted
    pub fn corrupted() -> Error<T> {
        Error(Kind::CorruptedHeader)
    }

    /// Create a new `Error` representing the function ID is not registered
    pub fn no_id() -> Error<T> {
        Error(Kind::NoID)
    }
}
