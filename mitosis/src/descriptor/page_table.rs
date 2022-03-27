use crate::{
    kern_wrappers::mm::{PhyAddrType, VirtAddrType},
    linux_kernel_module,
};

use hashbrown::HashMap;

use os_network::bytes::BytesMut;

/// Record the mapping between the va and remote pa of a process
#[derive(Default)]
pub struct FlatPageTable(pub HashMap<VirtAddrType, PhyAddrType>);

impl os_network::serialize::Serialize for FlatPageTable {
    /// Serialization format:
    /// ```
    /// total entries <usize> | AddrType <-8 bytes-> | Remote Page <-16 bytes->
    /// | AddrType <-8 bytes-> | Remote Page <-16 bytes-> |, ...
    /// ```
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        if bytes.len() < self.serialization_buf_len() {
            crate::log::error!("failed to serialize: buffer space not enough");
            return false;
        }

        let mut cur = unsafe { bytes.truncate_header(0).unwrap() };
        let sz = unsafe { cur.memcpy_serialize_at(0, &self.0.len()).unwrap() };
        let mut cur = unsafe { bytes.truncate_header(sz).unwrap() };

        for (vaddr, paddr) in self.0.iter() {
            let sz0 = unsafe { cur.memcpy_serialize_at(0, &vaddr).unwrap() };
            let sz1 = unsafe { cur.memcpy_serialize_at(1, &paddr).unwrap() };
            cur = unsafe { bytes.truncate_header(sz0 + sz1).unwrap() };
        }
        return true;
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let mut res = HashMap::new();
        let mut count: usize = 0;
        let off = unsafe { bytes.memcpy_deserialize(&mut count)? };

        let mut cur = unsafe { bytes.truncate_header(off)? };

        for _ in 0..count {
            let mut virt: VirtAddrType = 0;
            let mut phy: PhyAddrType = 0;

            let sz0 = unsafe { cur.memcpy_deserialize_at(0, &mut virt)? };
            let sz1 = unsafe { cur.memcpy_deserialize_at(sz0, &mut phy)? };

            // TODO: we need to identify that it is remote
            res.insert(virt, phy);

            cur = unsafe { bytes.truncate_header(sz0 + sz1)? };
        }

        Some(FlatPageTable(res))
    }

    fn serialization_buf_len(&self) -> usize {
        (core::mem::size_of::<VirtAddrType>() + core::mem::size_of::<PhyAddrType>()) * self.0.len()
    }
}
