use super::Future;

pub trait Datagram<T: Future = Self>: Future {
    type IOResult = Self::Error;
    type AddressHandler;
    type MemoryRegion;

    fn post_msg(
        &mut self,
        addr: &Self::AddressHandler,
        msg: &Self::MemoryRegion,
    ) -> Result<(), Self::IOResult>;

    // TODO: should be move to another trait
    fn post_recv_buf(&mut self, buf: Self::MemoryRegion) -> Result<(), Self::IOResult>;
}

pub trait Receiver<D, T: Future = Self>: Future where D : Datagram {
    type MsgBuf = D::MemoryRegion; 

    fn post_recv();
}

pub trait Factory { 
    type DatagramType<'a>: Datagram
    where
        Self: 'a;
    type CreateMeta;
    type CreateResult;

    fn create(&self, meta: Self::CreateMeta) -> Result<Self::DatagramType<'_>, Self::CreateResult>;
}