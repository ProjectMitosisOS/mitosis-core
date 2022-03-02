#![no_std]
#![feature(generic_associated_types)]

extern crate alloc;

/// Reporter can be used to count the operation and report the result
pub mod reporter; 

/// Bench provides utilities to run mass parallel benchmarks
pub mod bench;

/// Ctx provides thread-local context abstraction in benchmarks
pub mod ctx;
