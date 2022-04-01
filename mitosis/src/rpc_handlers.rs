use crate::{get_descriptor_pool_ref, linux_kernel_module};
use os_network::bytes::BytesMut;
use core::fmt::Write;
use os_network::serialize::Serialize;
use crate::descriptors::{DescriptorFactoryService, ReadMeta};

#[derive(Debug)]
#[repr(usize)]
pub enum RPCId {
    // for test only
    Nil = 1,
    // for test only
    Echo = 2,
    // Resume fork by fetching remote descriptor
    ForkResume = 3,
}

pub mod payload {
    pub struct ForkResume {
        pub handler_id: usize,
        pub auth_key: usize,
    }

    impl os_network::serialize::Serialize for ForkResume {}
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
pub(crate) fn handle_fork_resume(input: &BytesMut, output: &mut BytesMut) -> usize {
    use payload::ForkResume;
    let fork_resume = ForkResume::deserialize(input);
    if fork_resume.is_none() {
        return 0;
    }
    let fork_resume = fork_resume.unwrap();
    let (handler_id, auth_key): (usize, usize) =
        (fork_resume.handler_id, fork_resume.auth_key);
    crate::log::debug!("[handle_swap_descriptor] start. handler id:{}, auth_key:{}", handler_id, auth_key);
    // 1. Read from descriptor pool
    let dfs: &DescriptorFactoryService = unsafe { get_descriptor_pool_ref() };
    if let Some(meta) = dfs.get_descriptor_ref(handler_id) {
        // 2. Write back
        let dst = ReadMeta {
            addr: dfs.get_descriptor_dma_buf(handler_id).unwrap(),
            length: meta.serialization_buf_len() as _,
        };
        output.resize(dst.serialization_buf_len());
        return match dst.serialize(output) {
            true => output.len(),
            false => {
                crate::log::error!("fail to serialize descriptor. handler id:{}, auth key:{}",
                handler_id, auth_key);
                0
            }
        };
    }
    return 0;
}
