use super::Future;

/// A trait for connection: A connection should support posting (submiting) requests and polling results from trait `Future`.
///
pub trait Conn<T: Future = Self>: Future {
    type ReqPayload; // the request format
    type CompPayload = Self::Output;
    type IOResult = Self::Error;

    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult>;
}

/// A general trait to generate connection object.
///
pub trait Factory {
    type ConnMeta;
    type ConnType;
    type ConnResult;

    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType, Self::ConnResult>;
}

/// A general trait to generate connection metadata.
///
pub trait MetaFactory: Factory {
    type HyperMeta: Default;
    type Meta;
    type MetaResult;

    fn create_meta(&self, meta: Self::HyperMeta) -> Result<Self::Meta, Self::MetaResult>;
}
