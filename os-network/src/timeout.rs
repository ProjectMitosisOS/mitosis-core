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

use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util::timer::KTimer;

pub struct Delay {
    timer: KTimer,
    wait_usec: i64,
}

impl Delay {
    pub fn new(wait_usec: i64) -> Self {
        Self {
            timer: KTimer::new(),
            wait_usec: wait_usec,
        }
    }

    pub fn reset(&mut self) { 
        self.timer.reset(); 
    }

    pub fn reset_timer(&mut self, wait_usec : i64) { 
        self.wait_usec = wait_usec; 
        self.reset(); 
    }

    #[inline]
    pub fn get_cur_delay_usec(&self) -> i64 { 
        self.timer.get_passed_usec() 
    }
}

impl Future for Delay {
    type Output = i64;
    type Error = (); // return the elapsed time

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        let passed = self.get_cur_delay_usec(); 
        if passed >= self.wait_usec {
            return Ok(Async::Ready(passed));
        }
        return Ok(Async::NotReady);
    }
}

pub struct Timeout<T> {
    value: T,
    delay: Delay,
}

pub struct TimeoutWRef<'a, T> { 
    value : &'a mut T, 
    delay : Delay,
}

impl<T> Timeout<T> {
    /// create a Timeout to wrap a future
    pub fn new(value: T, timeout_usec: i64) -> Timeout<T> {
        Self {
            value: value,
            delay: Delay::new(timeout_usec),
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

    pub fn get_inner_mut(&mut self) -> &mut T { 
        &mut self.value
    }

    /// Reset the counting of the timeout 
    pub fn reset_timer(&mut self, timeout_usec : i64) { 
        self.delay.reset_timer(timeout_usec); 
    }

    /// Get current wait status 
    pub fn get_cur_delay_usec(&self) -> i64 { 
        self.delay.get_cur_delay_usec() 
    }    
}

impl<'a, T> TimeoutWRef<'a, T> {
    /// create a Timeout to wrap a future
    pub fn new(value: &'a mut T, timeout_usec: i64) -> TimeoutWRef<'a, T> {
        Self {
            value: value,
            delay: Delay::new(timeout_usec),
        }
    }
}

impl<T> Future for Timeout<T>
where
    T: Future,
{
    type Output = T::Output; // result, passed_msec
    type Error = Error<T::Error>;

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        // First, try polling the future
        match self.value.poll() {
            Ok(Async::Ready(v)) => {
                return Ok(Async::Ready(v));
            }
            Ok(Async::NotReady) => {}
            Err(e) => return Err(Error::inner(e)),
        }

        // Now check the timer
        match self.delay.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(_passed)) => {
                return Err(Error::elapsed()); 
            }
            Err(_) => panic!(),
        }
    }
}

impl<'a, T> Future for TimeoutWRef<'a, T>
where
    T: Future,
{
    type Output = T::Output; // result, passed_msec
    type Error = Error<T::Error>;

    fn poll(&mut self) -> Poll<Self::Output, Self::Error> {
        // First, try polling the future
        match self.value.poll() {
            Ok(Async::Ready(v)) => {
                return Ok(Async::Ready(v));
            }
            Ok(Async::NotReady) => {}
            Err(e) => return Err(Error::inner(e)),
        }

        // Now check the timer
        match self.delay.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(_passed)) => {
                return Err(Error::elapsed()); 
            }
            Err(_) => panic!(),
        }
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
