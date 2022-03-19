use super::Future;

pub trait Receiver<T: Future = Self>: Future
{
    type IOResult = Self::Error;
    type MsgBuf;
    const Header : usize; 

    fn post_recv_buf(&mut self, buf: Self::MsgBuf) -> Result<(), Self::IOResult>;
}

pub mod msg;
pub mod ud;
pub mod ud_receiver;
