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

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct DescriptorLookupReply {
    pub(crate) pa: u64,
    pub(crate) sz: usize,
    pub(crate) rkey: u32,
    pub(crate) ready: bool,
}

impl os_network::serialize::Serialize for DescriptorLookupReply {}

pub(crate) fn handle_descriptor_addr_lookup(input: &BytesMut, output: &mut BytesMut) -> usize {
    let mut key: usize = 0;
    unsafe { input.memcpy_deserialize(&mut key) };

    let process_service = unsafe { crate::get_sps_mut() };
    let buf = process_service.query_descriptor_buf(key);

    if buf.is_none() {
        crate::log::error!("empty addr, key:{}!", key);

        // send an error reply
        return 0; // a null reply indicate that the we don't have the key
    }

    let reply = match buf {
        Some((addr, len, rkey)) => {
            DescriptorLookupReply {
                pa: addr.get_pa(),
                sz: len,
                rkey: rkey,
                ready: true,
            }
        }
        None => {
            crate::log::error!("Failed to find the handner with id: {}!", key);
            DescriptorLookupReply {
                pa: 0,
                sz: 0,
                rkey: 0,
                ready: false,
            }
        }
    };

    reply.serialize(output);
    reply.serialization_buf_len()
}
