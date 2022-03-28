use crate::linux_kernel_module;
use os_network::bytes::BytesMut;
use core::fmt::Write;
use os_network::serialize::Serialize;
use crate::descriptors::{DescriptorFactoryService};

#[derive(Debug)]
#[repr(usize)]
pub enum RPCId {
    // for test only
    Nil = 1,
    // for test only
    Echo = 2,
    // Fetch parent's container descriptor
    SwapDescriptor = 3,
}

pub(crate) fn handle_nil(_input: &BytesMut, _output: &mut BytesMut) -> usize {
    64
}


pub(crate) fn handle_echo(input: &BytesMut, output: &mut BytesMut) -> usize {
    crate::log::info!("echo callback {:?}", input);
    write!(output, "Hello from MITOSIS").unwrap();
    64
}

/// Handler for fetching parent's descriptor. The payload should contains
/// `handler_id` and `auth_key` for query.
///
/// The reply of the `Descriptor` would send back the specific start physical address and length (for one-sided read)
pub(crate) fn handle_swap_descriptor(_input: &BytesMut, output: &mut BytesMut) -> usize {
    let (handler_id, auth_key): (usize, usize) = (0x0, 0x1);
    crate::log::debug!("[handle_swap_descriptor] start. handler id:{}, auth_key:{}", handler_id, auth_key);
    // 1. Read from descriptor pool
    let dfs: DescriptorFactoryService = DescriptorFactoryService::create(); // TODO: get from global var
    if let Some(meta) = dfs.get_descriptor(0) {
        // 2. Write back
        return match meta.serialize(output) {
            true => {
                crate::log::debug!("find one meta: {:?}", output);
                meta.serialization_buf_len() as _
            }
            false => {
                crate::log::error!("fail to serialize descriptor. handler id:{}, auth key:{}",
                handler_id, auth_key);
                0
            }
        };
    }
    return 0;
}
