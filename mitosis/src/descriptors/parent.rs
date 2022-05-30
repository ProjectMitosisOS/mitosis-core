use crate::descriptors::{ChildDescriptor, RDMADescriptor, RegDescriptor, VMADescriptor};
use crate::kern_wrappers::mm::{PhyAddrType, VirtAddrType};
use crate::{linux_kernel_module, VmallocAllocator};
use alloc::vec::Vec;
use os_network::bytes::BytesMut;
use os_network::serialize::Serialize;

#[cfg(feature = "prefetch")]
use crate::prefetcher::DCAsyncPrefetcher;

pub(crate) type Offset = u32;
pub(crate) type Value = PhyAddrType;
type PageEntry = (Offset, Value); // record the (offset, phy_addr) pair

/// This is a simple, condensed page table to represent the parent's
/// page table in the descriptor.
#[derive(Clone)]
pub struct CompactPageTable {
    inner_pg_table: Vec<PageEntry, VmallocAllocator>,
}

/// The descriptor used at the parents
#[allow(dead_code)]
#[derive(Clone)]
pub struct ParentDescriptor {
    pub regs: RegDescriptor,
    // 2-dimension matrix, each row means one page-table according to one VMA
    pub page_table: Vec<CompactPageTable, VmallocAllocator>,
    pub vma: Vec<VMADescriptor>,
    pub machine_info: RDMADescriptor,
}

impl Default for ParentDescriptor {
    fn default() -> Self {
        Self {
            regs: Default::default(),
            page_table: Vec::new_in(VmallocAllocator),
            vma: Vec::new(),
            machine_info: Default::default(),
        }
    }
}

impl ParentDescriptor {
    /// Transform the parent descriptor to a child descriptor
    /// When prefetch is enabled, this function can fail if no sufficient 
    /// DCQP in the current kernel. 
    #[inline]
    pub fn to_descriptor(&self) -> ChildDescriptor {
        #[cfg(feature = "prefetch")]
        let mut page_table = crate::remote_mapping::RemotePageTable::new();

        #[cfg(not(feature = "prefetch"))]
        let mut page_table = super::page_table::FlatPageTable::new();

        for (vma_idx, vma_pg_table) in self.page_table.iter().enumerate() {
            let start = self.vma[vma_idx].get_start();
            for (offset, phy_addr) in &vma_pg_table.inner_pg_table {
                #[cfg(not(feature = "prefetch"))]
                page_table.add_one((*offset as VirtAddrType + start) as _, *phy_addr as _);

                #[cfg(feature = "prefetch")]
                page_table.map(
                    crate::remote_mapping::VirtAddr::new(*offset as VirtAddrType + start),
                    crate::remote_mapping::PhysAddr::new(*phy_addr),
                );
            }
        }

        #[cfg(feature = "prefetch")]
        let (prefetch_conn, lkey) = unsafe {
            crate::get_dc_pool_async_service_ref()
                .lock(|p| p.pop_one_qp())
                .expect("failed to create prefetcher")
        };

        #[cfg(feature = "prefetch")]
        let access_info = crate::remote_paging::AccessInfo::new(&self.machine_info).unwrap();

        ChildDescriptor {
            regs: self.regs.clone(),
            page_table,
            vma: self.vma.clone(),
            machine_info: self.machine_info.clone(),

            #[cfg(feature = "prefetch")]
            prefetcher: DCAsyncPrefetcher::new_from_raw(prefetch_conn, lkey, access_info),
        }
    }
}

impl ParentDescriptor {
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

impl os_network::serialize::Serialize for CompactPageTable {
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
        let sz = unsafe {
            cur.memcpy_serialize_at(0, &self.inner_pg_table.len())
                .unwrap()
        };
        cur = unsafe { cur.truncate_header(sz).unwrap() };
        if core::mem::size_of::<Offset>() < core::mem::size_of::<VirtAddrType>()
            && self.table_len() % 2 == 1
        {
            let pad: u32 = 0;
            let sz = unsafe { cur.memcpy_serialize_at(0, &pad).unwrap() };
            cur = unsafe { cur.truncate_header(sz).unwrap() };
        }

        for (offset, paddr) in self.inner_pg_table.iter() {
            let sz0 = unsafe { cur.write_unaligned_at_head(*offset) };
            cur = unsafe { cur.truncate_header(sz0).unwrap() };

            let sz0 = unsafe { cur.write_unaligned_at_head(*paddr) };
            cur = unsafe { cur.truncate_header(sz0).unwrap() };
        }

        true
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let mut res: Vec<PageEntry, VmallocAllocator> = Vec::new_in(VmallocAllocator);
        let mut count: usize = 0;
        let mut cur = unsafe { bytes.truncate_header(0).unwrap() };

        let off = unsafe { cur.memcpy_deserialize(&mut count)? };

        cur = unsafe { cur.truncate_header(off)? };

        if core::mem::size_of::<Offset>() < core::mem::size_of::<VirtAddrType>() && count % 2 == 1 {
            let mut pad: u32 = 0;
            let off = unsafe { cur.memcpy_deserialize(&mut pad)? };
            cur = unsafe { cur.truncate_header(off)? };
        }

        for _ in 0..count {
            let virt: Offset = unsafe { cur.read_unaligned_at_head() };
            cur = unsafe { cur.truncate_header(core::mem::size_of::<Offset>())? };

            let phy: Value = unsafe { cur.read_unaligned_at_head() };
            cur = unsafe { cur.truncate_header(core::mem::size_of::<Value>())? };

            res.push((virt, phy));
        }

        Some(CompactPageTable {
            inner_pg_table: res,
        })
    }

    fn serialization_buf_len(&self) -> usize {
        let mut base = core::mem::size_of::<usize>()
            + self.inner_pg_table.len()
                * (core::mem::size_of::<Offset>() + core::mem::size_of::<Value>());
        if core::mem::size_of::<Offset>() < core::mem::size_of::<VirtAddrType>()
            && self.table_len() % 2 == 1
        {
            base += core::mem::size_of::<u32>();
        }
        base
    }
}

impl os_network::serialize::Serialize for ParentDescriptor {
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
        let mut cur = unsafe {
            // update cursor
            cur.truncate_header(self.regs.serialization_buf_len())
                .unwrap()
        };

        // 2. vmas & page table (size)
        let sz = unsafe { cur.memcpy_serialize_at(0, &self.page_table.len()).unwrap() };
        let mut cur = unsafe { cur.truncate_header(sz).unwrap() };
        assert_eq!(self.vma.len(), self.page_table.len());

        //   page table (vec)
        for (i, vma_pg_table) in self.page_table.iter().enumerate() {
            let vma = self.vma[i];
            vma.serialize(&mut cur);
            cur = unsafe { cur.truncate_header(vma.serialization_buf_len()).unwrap() };

            vma_pg_table.serialize(&mut cur);
            cur = unsafe {
                cur.truncate_header(vma_pg_table.serialization_buf_len())
                    .unwrap()
            };
        }

        // 3. finally, machine info
        self.machine_info.serialize(&mut cur);

        true
    }

    /// **TODO**
    /// - Currently, we don't check the buf len, so this function is **unsafe**
    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let mut cur = unsafe { bytes.truncate_header(0).unwrap() };
        // regs
        let regs = RegDescriptor::deserialize(&cur)?;
        cur = unsafe { cur.truncate_header(regs.serialization_buf_len())? };

        // VMA page counts
        let mut count: usize = 0;
        let off = unsafe { cur.memcpy_deserialize(&mut count)? };
        cur = unsafe { cur.truncate_header(off)? };

        // VMA & its corresponding page table
        let mut pt = Vec::new_in(VmallocAllocator);
        let mut vmas = Vec::new();

        for _ in 0..count {
            let vma = VMADescriptor::deserialize(&cur)?;
            cur = unsafe { cur.truncate_header(vma.serialization_buf_len())? };
            vmas.push(vma);

            let vma_pg_table = CompactPageTable::deserialize(&cur)?;
            cur = unsafe { cur.truncate_header(vma_pg_table.serialization_buf_len())? };
            pt.push(vma_pg_table);
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

impl Default for CompactPageTable {
    fn default() -> Self {
        Self {
            inner_pg_table: Vec::new_in(VmallocAllocator),
        }
    }
}

impl CompactPageTable {
    #[inline(always)]
    pub fn add_one(&mut self, offset: Offset, val: Value) {
        self.inner_pg_table.push((offset, val))
    }

    #[inline(always)]
    pub fn table_len(&self) -> usize {
        self.inner_pg_table.len()
    }
}
