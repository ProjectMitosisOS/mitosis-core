/// A simple async marker, inspired by tokio
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Async<T> {
    /// Represents that a value is immediately ready.
    Ready(T),

    /// Represents that a value is not ready yet, but may be so later.
    NotReady,
}

impl<T> From<T> for Async<T> {
    fn from(t: T) -> Async<T> {
        Async::Ready(t)
    }
}

pub type Poll<T, E> = Result<Async<T>, E>;

/// a simplified in-kernel future
/// inspired by tokio 
pub trait Future {
    type Output;
    type Error;

    fn poll<'a>(&'a mut self) -> Poll<Self::Output, Self::Error>; 
}

