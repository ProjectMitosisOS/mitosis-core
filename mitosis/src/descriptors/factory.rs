use hashbrown::HashMap;
use os_network::bytes::BytesMut;
use os_network::KRdmaKit::mem::Memory;
use os_network::serialize::Serialize;
use crate::descriptors::{Descriptor, RDMADescriptor};
use crate::kern_wrappers::mm::{PhyAddrType, VirtAddrType};
use crate::kern_wrappers::page::{KPageTable, Page};
use crate::kern_wrappers::Process;
use crate::KRdmaKit::consts::MAX_KMALLOC_SZ;
use crate::KRdmaKit::mem::RMemPhy;


/// A data structure for RPC to lookup the descriptor
/// It should be initialized in startup.rs
pub struct DescriptorFactoryService {
    // TODO: shall we wrap it into a lock? since there may be multiple RPC threads
    // TODO: shall we record the serialized buffer to avoid on-the-fly serialization? 
    registered_descriptors: HashMap<usize, super::Descriptor>,
    copy_pages: HashMap<usize, KPageTable>,
    // Map from `descriptor key` to `buffer physical address`
    registered_buf_table: HashMap<usize, PhyAddrType>,
    // Buffer of descriptors after serialization
    buf: Option<RMemPhy>,
}

impl DescriptorFactoryService {
    pub fn create() -> Self {
        Self {
            // TODO: Lock maybe time consuming ? => do not lock on read
            registered_descriptors: Default::default(),
            copy_pages: Default::default(),
            registered_buf_table: Default::default(),
            buf: None,
        }
    }

    #[inline(always)]
    pub fn get_descriptor_ref(&self, key: usize) -> Option<&Descriptor> {
        self.registered_descriptors.get(&key)
    }

    #[inline(always)]
    pub fn get_descriptor_mut(&mut self, key: usize) -> Option<&mut Descriptor> {
        self.registered_descriptors.get_mut(&key)
    }

    #[inline(always)]
    pub fn get_descriptor_dma_buf(&self, key: usize) -> Option<&PhyAddrType> {
        self.registered_buf_table.get(&key)
    }


    #[inline(always)]
    pub fn put_current_descriptor(
        &mut self,
        key: usize,
        machine_info: RDMADescriptor) -> Option<&Descriptor> {
        let process = Process::new_from_current();
        let task = process.get_task();
        let (vma, pg_table) = task.generate_mm();
        let mut res = Descriptor {
            regs: task.generate_reg_descriptor(),
            page_table: Default::default(),
            vma,
            machine_info,
        };
        if self.get_descriptor_ref(key).is_some() {
            // should not assign twice
            None
        } else {
            // setup kpages
            // TODO: replace into CoW setting
            {
                let mut kpage: KPageTable = Default::default();
                for (addr, _) in &pg_table.0 {
                    let kp = unsafe { Page::new_from_upage(*addr as *mut _) }.unwrap();
                    let phy_addr = kp.get_phy();
                    kpage.insert(*addr, kp);
                    res.page_table.add_one(*addr, phy_addr); // refill the page table content
                }
                self.copy_pages.insert(key, kpage);
            }
            self.registered_descriptors.insert(key, res);
            // put buffer and map
            let mut buf = RMemPhy::new(MAX_KMALLOC_SZ);
            let mut tmp_buf_table: HashMap<usize, PhyAddrType> = Default::default();
            let mut start_pa = buf.get_pa(0) as PhyAddrType;
            let mut start_va = buf.get_ptr() as VirtAddrType;
            // TODO: optimized into ring buffer
            for (key, descriptor) in &self.registered_descriptors {
                let seri_len = descriptor.serialization_buf_len() as u64;
                let mut out = unsafe { BytesMut::from_raw(start_va as _, seri_len as _) };
                descriptor.serialize(&mut out);
                tmp_buf_table.insert(*key, start_pa);
                (start_va, start_pa) = (start_va + seri_len, start_pa + seri_len);
            }
            self.buf = Some(buf);
            self.registered_buf_table = tmp_buf_table.clone();
            self.get_descriptor_ref(key)
        }
    }

}