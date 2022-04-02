use hashbrown::HashMap;

use os_network::bytes::{ToBytes};
use os_network::serialize::Serialize;
use os_network::msg::UDMsg as RMemory;

use crate::descriptors::{Descriptor, RDMADescriptor};
use crate::kern_wrappers::mm::{PhyAddrType};
use crate::kern_wrappers::page::{KPageTable, Page};
use crate::kern_wrappers::Process;

/// A data structure for RPC to lookup the descriptor
/// It should be initialized in startup.rs
pub struct DescriptorFactoryService {
    // TODO: shall we wrap it into a lock? since there may be multiple RPC threads
    // TODO: shall we record the serialized buffer to avoid on-the-fly serialization? 
    registered_descriptors: HashMap<usize, super::Descriptor>,
    copy_pages: HashMap<usize, KPageTable>,
    // Local memory hash
    buf_hash: HashMap<usize, RMemory>,
}

impl DescriptorFactoryService {
    pub fn create() -> Self {
        Self {
            // TODO: Lock maybe time consuming ? => do not lock on read
            registered_descriptors: Default::default(),
            copy_pages: Default::default(),
            buf_hash: Default::default(),
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
    pub fn get_descriptor_dma_buf(&self, key: usize) -> Option<PhyAddrType> {
        if let Some(rmem) = self.buf_hash.get(&key) {
            Some(rmem.get_pa())
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn put_current_descriptor(
        &mut self,
        key: usize,
        machine_info: RDMADescriptor) -> Option<&Descriptor> {
        let process = Process::new_from_current();
        let task = process.get_task();
        let (vma, pg_table) = task.generate_mm();
        let mut descriptor_meta = Descriptor {
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
                    descriptor_meta.page_table.add_one(*addr, phy_addr); // refill the page table content
                }
                self.copy_pages.insert(key, kpage);
            }
            // put buffer and map
            let mut buf = RMemory::new(descriptor_meta.serialization_buf_len(), 0);
            descriptor_meta.serialize(buf.get_bytes_mut());
            self.buf_hash.insert(key, buf);
            self.registered_descriptors.insert(key, descriptor_meta);
            self.get_descriptor_ref(key)
        }
    }
}