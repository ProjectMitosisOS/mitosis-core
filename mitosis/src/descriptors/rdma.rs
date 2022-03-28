use os_network::rdma::RawGID;

#[allow(dead_code)]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct RDMADescriptor {
    // TODO: these fields are not correct.
    // The design of MITOSIS requires client using no-roundtrip to fork from the parent.
    // However, CM still uses one RTT
    pub(crate) gid: RawGID,
    pub(crate) service_id: u64,
    pub(crate) rkey: u32,
}

impl RDMADescriptor {
    pub fn get_rkey(&self) -> u32 {
        self.rkey
    }

    pub fn get_service_id(&self) -> u64 {
        self.service_id
    }

    pub fn set_rkey(&mut self, key: u32) -> &mut Self {
        self.rkey = key;
        self
    }

    pub fn set_service_id(&mut self, id: u64) -> &mut Self {
        self.service_id = id;
        self
    }
}

impl os_network::serialize::Serialize for RDMADescriptor {}

