use crate::linux_kernel_module;
use core::fmt::Write;
use os_network::bytes::BytesMut;
use os_network::serialize::Serialize;

#[derive(Debug)]
#[repr(usize)]
pub enum RPCId {
    // for test only
    Nil = 1,
    // for test only
    Echo = 2,
    // Resume fork by fetching remote descriptor
    Query = 3,
}

pub(crate) fn handle_nil(_input: &BytesMut, _output: &mut BytesMut) -> usize {
    64
}

pub(crate) fn handle_echo(input: &BytesMut, output: &mut BytesMut) -> usize {
    crate::log::info!("echo callback {:?}", input);
    write!(output, "Hello from MITOSIS").unwrap();
    64
}

#[derive(Debug,Default)]
pub(crate) struct DescriptorLookupReply {
    pub(crate) pa: u64,
    pub(crate) sz: usize,
}

impl os_network::serialize::Serialize for DescriptorLookupReply {}

pub(crate) fn handle_descriptor_addr_lookup(input: &BytesMut, output: &mut BytesMut) -> usize {
    let mut key: usize = 0;
    unsafe { input.memcpy_deserialize(&mut key) };

    let process_service = unsafe { crate::get_sps_mut() };
    let addr = process_service.query_descriptor_buf(key);

    if addr.is_none() {
        crate::log::error!("empty addr, key:{}!", key);
        return 0; // a null reply indicate that the we don't have the key
    }

    let reply = DescriptorLookupReply {
        pa : addr.unwrap().get_pa(),
        sz : addr.unwrap().len(),
    };
    crate::log::debug!("send reply:{:?}", reply);
    reply.serialize(output);
    reply.serialization_buf_len()
}
