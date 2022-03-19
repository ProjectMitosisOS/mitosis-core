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
pub mod serialize;
pub mod remote_memory;

/// Connection abstraction for RC and DC
pub mod conn;
pub use conn::*;
/// TODO: need doc
pub mod datagram;
pub use datagram::*;

pub mod rpc;

// impl the connection as RDMA
pub mod rdma;

#[allow(unused_imports)]
use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util as log;
use KRdmaKit::rust_kernel_rdma_base::rust_kernel_linux_util;

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
