use super::Future;

pub trait Conn<T: Future = Self>: Future {
    type ReqPayload; // the request format
    type CompPayload = Self::Output;
    type IOResult = Self::Error;

    // post the request to the underlying device
    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult>;
}

pub trait Factory {
    type ConnMeta;
    type ConnType<'a>: Conn
    where
        Self: 'a;
    type ConnResult;

    // create and connect the connection
    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType<'_>, Self::ConnResult>;
}

pub trait MetaFactory: Factory {
    type HyperMeta;
    type Meta;
    type MetaResult;

    fn create_meta(&self, meta: Self::HyperMeta) -> Result<Self::Meta, Self::MetaResult>;
}
