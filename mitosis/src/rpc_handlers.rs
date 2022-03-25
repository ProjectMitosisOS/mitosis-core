use crate::linux_kernel_module;
use os_network::bytes::BytesMut;

#[derive(Debug)]
#[repr(usize)]
pub enum RPCId {
    Nil = 1,  // for test only
    Echo = 2, // for test only
}

pub(crate) fn handle_nil(_input: &BytesMut, _output: &mut BytesMut) -> usize {
    64
}

use core::fmt::Write;

pub(crate) fn handle_echo(input: &BytesMut, output: &mut BytesMut) -> usize {
    crate::log::info!("echo callback {:?}", input);
    write!(output, "Hello from MITOSIS").unwrap();
    64
}
