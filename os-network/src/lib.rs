#![no_std]
#![feature(
    generic_associated_types,
    get_mut_unchecked,
    core_intrinsics,
    associated_type_defaults
)]

// Manage network connections in the OS

extern crate alloc;

pub mod future;

pub use future::Future;

pub mod timeout;

pub mod bytes;
pub mod remote_memory;

pub trait Conn<T: Future = Self>: Future {
    type ReqPayload; // the request format
    type CompPayload = Self::Output;
    type IOResult = Self::Error;

    // post the request to the underlying device
    fn post(&mut self, req: &Self::ReqPayload) -> Result<(), Self::IOResult>;
}

pub trait Datagram<T: Future = Self>: Future {
    type IOResult = Self::Error;
    type AddressHandler;
    type MemoryRegion;

    fn post_msg(
        &mut self,
        addr: &Self::AddressHandler,
        msg: &Self::MemoryRegion,
    ) -> Result<(), Self::IOResult>;

    fn post_recv_buf(&mut self, buf: Self::MemoryRegion) -> Result<(), Self::IOResult>;
}

pub trait ConnFactory {
    type ConnMeta;
    type ConnType<'a>: Conn
    where
        Self: 'a;
    type ConnResult;

    // create and connect the connection
    fn create(&self, meta: Self::ConnMeta) -> Result<Self::ConnType<'_>, Self::ConnResult>;
}

// impl the connection as RDMA
pub mod rdma;

/// Block on the future until it is ready or error
#[allow(non_snake_case)]
pub fn block_on<F: Future>(f: &mut F) -> Result<F::Output, F::Error> {
    use future::Async;
    loop {
        match f.poll() {
            Ok(Async::Ready(v)) => return Ok(v),
            Ok(_NotReady) => {}
            Err(e) => return Err(e),
        }
    }
}

/// Block on the future until it is ready or error
#[allow(non_snake_case)]
pub fn block_on_w_yield<F: Future>(f: &mut F) -> Result<F::Output, F::Error> {
    use future::Async;
    use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util::kthread::yield_now;

    loop {
        match f.poll() {
            Ok(Async::Ready(v)) => return Ok(v),
            Ok(_NotReady) => {}
            Err(e) => return Err(e),
        }

        yield_now();
    }
}
