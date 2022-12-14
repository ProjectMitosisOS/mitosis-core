use hashbrown::HashMap;

use super::*;
use crate::datagram::Receiver;
use crate::{bytes::ToBytes, conn::*};

/// The hook will receive datagram requests, and call the RPC callback correspondingly.
/// * Factory: the connection factory that creates the session
/// * R: Receiver
pub struct RPCHook<'a, F, R, MF>
where
    F: RPCFactory,
    R: Receiver,
{
    service: Service<'a>,
    session_factory: F,
    meta_factory: Arc<MF>,
    transport: R,
    connected_sessions: HashMap<usize, (F::ConnType, R::MsgBuf)>,

    // counting data
    analysis: super::analysis::RPCAnalysis,
}

impl<'a, F, R, MF> RPCHook<'a, F, R, MF>
where
    F: RPCFactory,
    R: Receiver,
{
    pub fn get_mut_service(&mut self) -> &mut Service<'a> {
        &mut self.service
    }
}

use super::header_factory::*;

impl<'a, F, R, MF> RPCHook<'a, F, R, MF>
where
    F: RPCFactory,
    // we need to ensure that the polled result can be sent back to
    R: Receiver<
        Output = <<F as RPCFactory>::ConnType as RPCConn>::ReqPayload,
        MsgBuf = <<F as RPCFactory>::ConnType as RPCConn>::ReqPayload,
        IOResult = <R as Future>::Error,
    >,
    <<F as RPCFactory>::ConnType as RPCConn>::ReqPayload: ToBytes,
    <F as RPCFactory>::ConnType: crate::future::Future,
{
    pub fn new(meta_f: Arc<MF>, factory: F, transport: R) -> Self {
        Self {
            service: Service::new(),
            meta_factory: meta_f,
            session_factory: factory,
            connected_sessions: HashMap::new(),
            transport: transport,
            analysis: super::analysis::RPCAnalysis::new(),
        }
    }

    pub fn post_msg_buf(&mut self, msg: R::MsgBuf) -> Result<(), R::IOResult> {
        self.transport.post_recv_buf(msg)
    }

    pub fn get_analysis(&self) -> &super::analysis::RPCAnalysis {
        &self.analysis
    }

    fn send_reply(
        &mut self,
        session_id: usize,
        reply: ReplyStubFactory,
    ) -> Result<(), Error<R::Error>> {
        let (session, buf) = self
            .connected_sessions
            .get_mut(&session_id)
            .ok_or(Error::session_creation_error())?;

        let signal = session.get_pending_reqs() == 0;

        crate::log::debug!(
            "send a request with signal {} to session {}, pending: {}",
            signal,
            session_id,
            session.get_pending_reqs()
        );

        let msg_sz = reply.get_payload() + reply.generate(buf.get_bytes_mut()).unwrap(); // should succeed
        let ret = session
            .post(&buf, msg_sz, signal)
            .map_err(|_| Error::fatal());

        // FIXME: 4 is a magic number I used here
        // the outstanding request threshold should not be greater
        // to prevent ib_post_send failure
        if session.get_pending_reqs() > 4 {
            crate::block_on(session).map_err(|_| Error::fatal())?;
            assert!(session.get_pending_reqs() == 0);
        } 
        return ret;           
    }
}

#[allow(unused_imports)]
use super::header::*;

impl<'a, F, R, MF> Future for RPCHook<'a, F, R, MF>
where
    F: RPCFactory,
    // we need to ensure that the polled result can be sent back to
    R: Receiver<
        Output = <<F as RPCFactory>::ConnType as RPCConn>::ReqPayload,
        MsgBuf = <<F as RPCFactory>::ConnType as RPCConn>::ReqPayload,
        IOResult = <R as Future>::Error,
    > + GetContext<
        Context = <<<F as RPCFactory>::ConnType as RPCConn>::ReqPayload as AllocMsgBuf>::Context,
    >,
    <<F as RPCFactory>::ConnType as RPCConn>::ReqPayload: ToBytes,
    MF: MetaFactory<Meta = F::ConnMeta>,
{
    type Output = (); // msg, session ID
    type Error = Error<R::Error>;

    fn poll<'r>(&'r mut self) -> Poll<Self::Output, Self::Error> {
        match self.transport.poll() {
            // not receiving any request, just move on
            Ok(Async::NotReady) => Ok(Async::NotReady),

            // msg received
            Ok(Async::Ready(msg)) => {
                // parse the request
                let bytes = unsafe { msg.get_bytes().clone() };
                let mut msg_header: MsgHeader = Default::default();

                // the datagram may have a extra header (e.g., GRH_HEADER in UD)
                // so we must truncate it first
                let msg_header_bytes =
                    unsafe { bytes.truncate_header(R::HEADER).ok_or(Error::corrupted()) }?;
                unsafe { msg_header_bytes.memcpy_deserialize(&mut msg_header) };

                let rpc_args = unsafe {
                    msg_header_bytes
                        .truncate_header(core::mem::size_of::<super::header::MsgHeader>())
                        .and_then(|msg| msg.clone_and_resize(msg_header.get_payload()))
                        .ok_or(Error::corrupted())?
                };

                // parse and dispatch the message
                match msg_header.get_marker() {
                    super::header::ReqType::Connect => {
                        let meta = msg_header.get_connect_stub().ok_or(Error::corrupted())?;
                        //crate::log::debug!("handle in-coming connect req {:?}",meta);

                        // insert the session if necessary
                        if !self.connected_sessions.contains_key(&meta.get_session_id()) {
                            let mut session_meta: <MF as MetaFactory>::HyperMeta =
                                Default::default();
                            unsafe {
                                rpc_args
                                    .memcpy_deserialize(&mut session_meta)
                                    .ok_or(Error::corrupted())?
                            };
                            let connect_meta = self
                                .meta_factory
                                .create_meta(session_meta)
                                .map_err(|_| Error::session_creation_error())?;

                            let session = self
                                .session_factory
                                .create(connect_meta)
                                .map_err(|_| Error::session_creation_error())?;

                            let session_buf = R::MsgBuf::create(R::MTU, 0, self.transport.get_context());

                            // add to my connected session
                            self.connected_sessions
                                .insert(meta.get_session_id(), (session, session_buf));
                            // send the reply
                            self.send_reply(
                                meta.get_session_id(),
                                ReplyStubFactory::new(ReplyStatus::Ok, 0),
                            )?;
                            // crate::log::info!("send reply done");
                        } else {
                            crate::log::debug!(
                                "duplicate connect session ID: {}",
                                meta.get_session_id()
                            );
                            return Err(Error::session_creation_error());
                        }

                        // handle connect message done
                    }

                    // handle the RPC request
                    super::header::ReqType::Request => {
                        let meta = msg_header.get_call_stub().ok_or(Error::corrupted())?;

                        let (_, out_buf) = self
                            .connected_sessions
                            .get_mut(&meta.get_session_id())
                            .ok_or(Error::not_connected())?;

                        let mut out_buf = unsafe {
                            out_buf
                                .get_bytes_mut()
                                .truncate_header(core::mem::size_of::<super::header::MsgHeader>())
                                .unwrap()
                        };

                        // handle the RPC request
                        let reply_payload =
                            self.service
                                .execute(meta.get_rpc_id(), &rpc_args, &mut out_buf);

                        self.analysis.handle_one();
                        self.analysis.handle_session_call(meta.get_session_id());

                        match reply_payload {
                            Some(size) => self.send_reply(
                                meta.get_session_id(),
                                ReplyStubFactory::new(ReplyStatus::Ok, size),
                            )?,
                            None => self.send_reply(
                                meta.get_session_id(),
                                ReplyStubFactory::new(ReplyStatus::NotExist, 0),
                            )?,
                        }
                        // handle RPC request done
                    }

                    // handle the session dis-connect
                    super::header::ReqType::DisConnect => {
                        // TODO @Yuhan: handle the dis connect part
                        todo!();
                    }

                    // Error case
                    _ => {
                        // should never happen at the hooker if no error happens !
                        crate::log::error!("unknown message header {:?}", msg_header);
                    }
                }

                // FIXME: Is the post_recv on the last ok? No.
                // Currently, if some error happens during the above process,
                // we may fail to post_recv_buf.
                // Will handle it later
                self.transport
                    .post_recv_buf(msg)
                    .map_err(|e| Error::inner(e))?;
                Ok(Async::Ready(()))
            }
            Err(e) => Err(Error::inner(e)),
        }
    }
}

use core::fmt::{Debug, Display, Formatter};

impl<'a, F, R, MF> Debug for RPCHook<'a, F, R, MF>
where
    F: RPCFactory,
    R: Receiver,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RPCHook\n  \t service: {}\n\t connected_sessions: {}, ncalls handled {}, other: {:?}",
            self.service,
            self.connected_sessions.len(),
            self.analysis.get_ncalls(),
            self.analysis.session_counts
        )
    }
}

impl<'a, F, R, MF> Display for RPCHook<'a, F, R, MF>
where
    F: RPCFactory,
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

    /// The session is not connected to the hook
    NotConnected,

    /// The header is corrupted
    CorruptedHeader,

    /// Failed to create the in-coming session
    SessionCreationError,

    /// The reply message exceeds the maximum msg buffers
    InValidSz,

    Fatal,
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

    pub fn session_creation_error() -> Error<T> {
        Error(Kind::SessionCreationError)
    }

    /// Create a new `Error` representing the function ID is not registered
    pub fn no_id() -> Error<T> {
        Error(Kind::NoID)
    }

    pub fn fatal() -> Error<T> {
        Error(Kind::Fatal)
    }

    pub fn invalid_sz() -> Error<T> {
        Error(Kind::InValidSz)
    }

    pub fn not_connected() -> Error<T> {
        Error(Kind::NotConnected)
    }
}
