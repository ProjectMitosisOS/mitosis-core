use hashbrown::HashMap;

use crate::shadow_process::*;

use os_network::{msg::UDMsg as RMemory, serialize::Serialize};
use os_network::bytes::ToBytes;

struct ProcessBundler {
    process: ShadowProcess,
    serialized_buf: RMemory,
}

impl ProcessBundler {
    fn new(process: ShadowProcess) -> Self {
        let mut buf = RMemory::new(process.get_descriptor_ref().serialization_buf_len(), 0);
        process.get_descriptor_ref().serialize(buf.get_bytes_mut());
        Self {
            process: process,
            serialized_buf: buf,
        }
    }
}

pub struct ShadowProcessService {
    registered_processes: HashMap<usize, ProcessBundler>,
}

impl ShadowProcessService {
    pub fn new() -> Self { 
        Self { 
            registered_processes : Default::default()
        }
    }
}
