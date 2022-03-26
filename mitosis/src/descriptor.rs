use alloc::vec::Vec;

use os_network::bytes::BytesMut;
use os_network::rdma::RawGID;

use hashbrown::HashMap;

type AddrType = u64;

/// Record the mapping between the va and remote pa of a process
type PageMap = HashMap::<AddrType, RemotePage>;

pub struct RemoteRDMADescriptor {
    gid: RawGID,
    service_id: u64,
    rkey: u32,
}

pub struct RemotePage {
    addr: AddrType,
    dct_key: u32,
}

pub struct VMADescriptor {
    range: (AddrType, AddrType), // [start,end] of a vma range
    flags: crate::bindings::vm_flags_t,
    prot: crate::bindings::pgprot_t,
}

#[allow(dead_code)]
pub struct Descriptor {
    regs: crate::bindings::PTRegs,
    page_table: PageMap,
    vma: Vec<VMADescriptor>,
    machine: RemoteRDMADescriptor,
    fs: u64,
    gs: u64,
}

impl os_network::serialize::Serialize for Descriptor {
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        unimplemented!();
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        unimplemented!();
    }
}
