use super::Future;

pub trait Datagram<T: Future = Self>: Future {
    type IOResult = Self::Error;
    type AddressHandler;
    type Msg;
    type Key;

    fn post_msg(
        &mut self,
        addr: &Self::AddressHandler,
        msg: &Self::Msg,
        key: &Self::Key,
    ) -> Result<(), Self::IOResult>;
}

pub trait Receiver<T: Future = Self>: Future
{
    type IOResult = Self::Error;
    type MsgBuf;
    type Key;

    fn post_recv_buf(&mut self, buf: Self::MsgBuf, key: Self::Key) -> Result<(), Self::IOResult>;
}

pub trait Factory {
    type DatagramType<'a>: Datagram
    where
        Self: 'a;
    type CreateMeta;
    type CreateResult;

    fn create(&self, meta: Self::CreateMeta) -> Result<Self::DatagramType<'_>, Self::CreateResult>;
}

pub mod msg;
pub mod ud;
pub mod ud_receiver;
