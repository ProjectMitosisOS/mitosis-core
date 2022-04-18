use alloc::vec::Vec;
use rust_kernel_linux_util::LevelFilter::Debug;
use os_network::bytes::BytesMut;
use os_network::serialize::Serialize;
use crate::descriptors::{Descriptor, FlatPageTable, RDMADescriptor, RegDescriptor, VMADescriptor};
use crate::kern_wrappers::mm::VirtAddrType;
// use crate::kern_wrappers::mm::{PhyAddrType, VirtAddrType};
use crate::linux_kernel_module;

type Offset = u32;
type Value = u32;
type PageEntry = (Offset, Value); // record the (offset, phy_addr) pair

#[derive(Default, Clone)]
pub struct VMAPageTable {
    pub inner_pg_table: Vec<PageEntry>,
}


#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct FastDescriptor {
    pub regs: RegDescriptor,
    // 2-dimension matrix, each row means one page-table according to one VMA
    pub page_table: Vec<VMAPageTable>,
    pub vma: Vec<VMADescriptor>,
    pub machine_info: RDMADescriptor,
}

impl FastDescriptor {
    /// Transform into the flat descriptor.
    #[inline]
    pub fn to_descriptor(&self) -> Descriptor {
        let mut page_table = FlatPageTable::new();

        let mut vma_idx = 0;
        for vma_pg_table in &self.page_table {
            let start = self.vma[vma_idx].get_start();
            for (offset, phy_addr) in &vma_pg_table.inner_pg_table {
                page_table.add_one((*offset as VirtAddrType + start) as _,
                                   *phy_addr as _);
            }
            vma_idx += 1;
        }

        Descriptor {
            regs: self.regs.clone(),
            page_table,
            vma: self.vma.clone(),
            machine_info: self.machine_info.clone(),
        }
    }
}

impl FastDescriptor {
    #[inline]
    fn vma_pg_table_serialization_buf_len(&self) -> usize {
        let mut result = core::mem::size_of::<usize>();
        // note that each vma offset-page-table may have different entry length !
        for vma_pg_table in &self.page_table {
            result += vma_pg_table.serialization_buf_len();
        }
        result
    }
}

impl os_network::serialize::Serialize for VMAPageTable {
    /// Serialization format:
    /// ```
    /// | inner_pg_table length in bytes <-8 bytes-> | inner_pg_table entries|
    /// ```
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        if bytes.len() < self.serialization_buf_len() {
            crate::log::error!(
                "failed to serialize: buffer space not enough. Need {}, actual {}",
                self.serialization_buf_len(),
                bytes.len()
            );
            return false;
        }
        let mut cur = unsafe { bytes.truncate_header(0).unwrap() };
        let sz = unsafe { cur.memcpy_serialize_at(0, &self.inner_pg_table.len()).unwrap() };
        let mut cur = unsafe { cur.truncate_header(sz).unwrap() };

        for (offset, paddr) in self.inner_pg_table.iter() {
            let sz0 = unsafe { cur.memcpy_serialize_at(0, offset).unwrap() };
            let sz1 = unsafe { cur.memcpy_serialize_at(sz0, paddr).unwrap() };
            cur = unsafe { cur.truncate_header(sz0 + sz1).unwrap() };
        }
        true
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let mut res: Vec<PageEntry> = Default::default();
        let mut count: usize = 0;
        let off = unsafe { bytes.memcpy_deserialize(&mut count)? };

        let mut cur = unsafe { bytes.truncate_header(off)? };
        for _ in 0..count {
            let mut virt: Offset = 0;
            let mut phy: Value = 0;

            let sz0 = unsafe { cur.memcpy_deserialize_at(0, &mut virt)? };
            let sz1 = unsafe { cur.memcpy_deserialize_at(sz0, &mut phy)? };

            res.push((virt, phy));

            cur = unsafe { cur.truncate_header(sz0 + sz1)? };
        }
        Some(VMAPageTable { inner_pg_table: res })
    }

    fn serialization_buf_len(&self) -> usize {
        core::mem::size_of::<usize>()
            + self.inner_pg_table.len() * core::mem::size_of::<PageEntry>()
    }
}

impl os_network::serialize::Serialize for FastDescriptor {
    /// Serialization format:
    /// ```
    /// | RegDescriptor <-sizeof(RegDescriptor)->
    /// | VMA page table length in bytes <-8 bytes-> | VMAPageMap
    /// | VMA descriptor length in bytes <-8 bytes-> | VMA descriptor
    /// | RDMADescriptor |
    /// ```
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        if bytes.len() < self.serialization_buf_len() {
            crate::log::error!(
                "failed to serialize: buffer space not enough. Need {}, actual {}",
                self.serialization_buf_len(),
                bytes.len()
            );
            return false;
        }

        // 1. Reg
        let mut cur = unsafe { bytes.truncate_header(0).unwrap() };
        self.regs.serialize(&mut cur);
        let mut cur = unsafe { // update cursor
            cur.truncate_header(self.regs.serialization_buf_len())
                .unwrap()
        };

        // 2. page table (size)
        let sz = unsafe { cur.memcpy_serialize_at(0, &self.page_table.len()).unwrap() };
        let mut cur = unsafe { cur.truncate_header(sz).unwrap() };
        //   page table (vec)
        for vma_pg_table in &self.page_table {
            vma_pg_table.serialize(&mut cur);
            cur = unsafe { cur.truncate_header(vma_pg_table.serialization_buf_len()).unwrap() };
        }


        // 3. vmas
        let sz = unsafe { cur.memcpy_serialize_at(0, &self.vma.len()).unwrap() };
        let mut cur = unsafe { cur.truncate_header(sz).unwrap() };

        for vma in &self.vma {
            vma.serialize(&mut cur);
            cur = unsafe { cur.truncate_header(vma.serialization_buf_len()).unwrap() };
        }
        // 4. finally, machine info
        self.machine_info.serialize(&mut cur);

        true
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let cur = unsafe { bytes.truncate_header(0).unwrap() };
        // regs
        let regs = RegDescriptor::deserialize(&cur)?;
        let cur = unsafe { cur.truncate_header(regs.serialization_buf_len())? };


        // vma pt
        let mut pt = Vec::new();
        {
            let mut count: usize = 0;
            let off = unsafe { cur.memcpy_deserialize(&mut count)? };
            let mut cur = unsafe { cur.truncate_header(off)? };

            for _ in 0..count {
                let vma_pg_table = VMAPageTable::deserialize(&cur)?;
                cur = unsafe { cur.truncate_header(vma_pg_table.serialization_buf_len())? };
                pt.push(vma_pg_table);
            }
        }
        // vmas
        let mut vmas = Vec::new();
        {
            let mut count: usize = 0;
            let off = unsafe { cur.memcpy_deserialize(&mut count)? };
            let mut cur = unsafe { cur.truncate_header(off)? };

            for _ in 0..count {
                let vma = VMADescriptor::deserialize(&cur)?;
                cur = unsafe { cur.truncate_header(vma.serialization_buf_len())? };
                vmas.push(vma);
            }
        }
        let machine_info = RDMADescriptor::deserialize(&cur)?;

        Some(Self {
            regs,
            page_table: pt,
            vma: vmas,
            machine_info,
        })
    }

    fn serialization_buf_len(&self) -> usize {
        self.regs.serialization_buf_len()
            + self.vma_pg_table_serialization_buf_len()
            + core::mem::size_of::<usize>() // the number of VMA descriptors
            + self.vma.len() * core::mem::size_of::<VMADescriptor>()
            + self.machine_info.serialization_buf_len()
    }
}

