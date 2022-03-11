// Credits: this piece of code is inspired by tokio

use crate::future::{Async, Future, Poll};

/// Error returned by `Timeout`.
#[derive(Debug)]
pub struct Error<T>(Kind<T>);

/// Timeout error variants
#[derive(Debug)]
enum Kind<T> {
    /// Inner value returned an error
    Inner(T),

    /// The timeout elapsed.
    Elapsed,
}

struct Delay {}

#[derive(Debug)]
pub struct Timeout<T> {
    value: T,
    delay_usec: usize,
}

impl<T> Timeout<T> {
    /// create a Timeout to wrap a future
    pub fn new(value: T, timeout_usec: usize) -> Timeout<T> {
        Self {
            value: value,
            delay_usec: timeout_usec,
        }
    }

    /// Gets a reference to the underlying value in this timeout.
    pub fn get_ref(&self) -> &T {
        &self.value
    }

    /// Gets a mutable reference to the underlying value in this timeout.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Consumes this timeout, returning the underlying value.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Future for Timeout<T>
where
    T: Future,
{
    type Item = T::Item;
    type Error = Error<T::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // First, try polling the future
        match self.value.poll() {
            Ok(Async::Ready(v)) => return Ok(Async::Ready(v)),
            Ok(Async::NotReady) => {}
            Err(e) => return Err(Error::inner(e)),
        }

        // Now check the timer
        /* the tokio code 
        match self.delay.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(_)) => Err(Error::elapsed()),
            Err(e) => Err(Error::timer(e)),
        } */
        todo!()
    }
}

// ===== impl Error ===== 
// Credits: from tokio 

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

    /// Consumes `self`, returning the inner future error.
    pub fn into_inner(self) -> Option<T> {
        match self.0 {
            Kind::Inner(err) => Some(err),
            _ => None,
        }
    }

    /// Create a new `Error` representing the inner value not completing before
    /// the deadline is reached.
    pub fn elapsed() -> Error<T> {
        Error(Kind::Elapsed)
    }

    /// Returns `true` if the error was caused by the inner value not completing
    /// before the deadline is reached.
    pub fn is_elapsed(&self) -> bool {
        match self.0 {
            Kind::Elapsed => true,
            _ => false,
        }
    }
}
