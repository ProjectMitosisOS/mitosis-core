use alloc::vec::Vec;

use os_network::bytes::BytesMut;
use os_network::rdma::RawGID;

use crate::linux_kernel_module;

use hashbrown::HashMap;

type AddrType = u64;

/// Record the mapping between the va and remote pa of a process
#[derive(Default)]
pub struct PageMap(pub HashMap<AddrType, RemotePage>);

impl os_network::serialize::Serialize for PageMap {
    /// Serialization format:
    /// ```
    /// | AddrType <-8 bytes-> | Remote Page <-16 bytes-> | AddrType <-8 bytes-> | Remote Page <-16 bytes-> |
    /// ```
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        if bytes.len() < self.serialization_len() {
            crate::log::error!("failed to serialize: buffer space not enough");
            return false;
        }
        let addr_size = core::mem::size_of::<AddrType>();
        let remote_page_size = core::mem::size_of::<RemotePage>();
        let mut serializer = unsafe { bytes.clone() };
        for (addr, remote_page) in self.0.iter() {
            // serialize addr
            serializer.copy(unsafe { &BytesMut::from_raw(addr as *const _ as *mut u8, addr_size) }, 0);
            let next = unsafe {
                serializer.truncate_header(addr_size)
            };
            if next.is_none() {
                crate::log::error!("failed to serialize");
                return false;
            }
            serializer = next.unwrap();

            // serialize remote_page
            serializer.copy(unsafe { &BytesMut::from_raw(remote_page as *const _ as *mut u8, remote_page_size) }, 0);
            let next = unsafe {
                serializer.truncate_header(remote_page_size)
            };
            if next.is_none() {
                crate::log::error!("failed to serialize");
                return false;
            }
            serializer = next.unwrap();
        }
        true
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let addr_size = core::mem::size_of::<AddrType>();
        let remote_page_size = core::mem::size_of::<RemotePage>();
        let entry_size = addr_size + remote_page_size;
        let count = bytes.len() / entry_size;
        let mut deserializer = unsafe { bytes.clone() };
        let mut page_map = HashMap::new();
        for _ in 0..count {
            let mut addr: AddrType = Default::default();
            let mut remote_page: RemotePage = Default::default();

            // deserialize addr
            unsafe {
                BytesMut::from_raw(&mut addr as *mut _ as *mut u8, addr_size)
            }.copy(unsafe { &deserializer.clone_and_resize(addr_size).unwrap() }, 0);
            deserializer = unsafe { deserializer.truncate_header(addr_size) }?;

            // deserialize remote_page
            unsafe {
                BytesMut::from_raw(&mut remote_page as *mut _ as *mut u8, remote_page_size)
            }.copy(unsafe { &deserializer.clone_and_resize(remote_page_size).unwrap() }, 0);
            deserializer = unsafe { deserializer.truncate_header(remote_page_size) }?;
            page_map.insert(addr, remote_page);
        }
        Some(PageMap(page_map))
    }

    fn serialization_len(&self) -> usize {
        let count = self.0.len();
        let entry_size = core::mem::size_of::<AddrType>() + core::mem::size_of::<RemotePage>();
        entry_size * count
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Descriptor {
    pub regs: RegDescriptor,
    pub page_table: PageMap,
    pub vma: Vec<VMADescriptor>,
    pub machine: RemoteRDMADescriptor,
}

pub mod reg;
use reg::RegDescriptor;

#[derive(Default)]
pub struct RemoteRDMADescriptor {
    pub gid: RawGID,
    pub service_id: u64,
    pub rkey: u32,
}

impl os_network::serialize::Serialize for RemoteRDMADescriptor {}

#[derive(Default, Copy, Clone, Debug)]
pub struct RemotePage {
    pub addr: AddrType,
    pub dct_key: u32,
}

impl os_network::serialize::Serialize for RemotePage {}

#[derive(Copy, Clone, Default)]
pub struct VMADescriptor {
    pub range: (AddrType, AddrType), // [start,end] of a vma range
    pub flags: crate::bindings::vm_flags_t,
    pub prot: crate::bindings::pgprot_t,
}

impl os_network::serialize::Serialize for VMADescriptor {}

impl os_network::serialize::Serialize for Descriptor {
    /// Serialization format:
    /// ```
    /// | RegDescriptor <-sizeof(RegDescriptor)-> | PageMap length in bytes <-8 bytes-> | PageMap | VMA descriptor length in bytes <-8 bytes-> | VMA descriptor | RemoteRDMADescriptor |
    /// ```
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        if bytes.len() < self.serialization_len() {
            crate::log::error!("failed to serialize: buffer space not enough");
            return false;
        }

        let mut serializer = unsafe {
            bytes.clone()
        };
        let reg_len = self.regs.serialization_len();
        let pagemap_len = self.page_table.serialization_len();
        let vma_len = self.vma.len() * core::mem::size_of::<VMADescriptor>();
        let machine_len = self.machine.serialization_len();

        // serialize regs
        let mut reg_bytes = unsafe {
            serializer.clone_and_resize(reg_len).unwrap()
        };
        self.regs.serialize(&mut reg_bytes);
        serializer = unsafe {
            serializer.truncate_header(reg_len).unwrap()
        };

        // serialize PageMap length (usize)
        unsafe {
            *(serializer.get_ptr() as *mut usize) = pagemap_len;
        }
        serializer = unsafe {
            serializer.truncate_header(core::mem::size_of::<usize>()).unwrap()
        };

        // serialize PageMap
        let mut page_map_bytes = unsafe {
            serializer.clone_and_resize(pagemap_len).unwrap()
        };
        self.page_table.serialize(&mut page_map_bytes);
        serializer = unsafe {
            serializer.truncate_header(pagemap_len).unwrap()
        };

        // serialize VMADescriptors' length (usize)
        unsafe {
            *(serializer.get_ptr() as *mut usize) = vma_len;
        }
        serializer = unsafe {
            serializer.truncate_header(core::mem::size_of::<usize>()).unwrap()
        };

        // serialize VMADescriptors
        let mut vma_descriptor_bytes = unsafe {
            serializer.clone_and_resize(vma_len).unwrap()
        };
        let raw_vm_descriptors = unsafe {
            BytesMut::from_raw(self.vma.as_ptr() as *mut u8, vma_len)
        };
        vma_descriptor_bytes.copy(&raw_vm_descriptors, 0);
        serializer = unsafe {
            serializer.truncate_header(vma_len).unwrap()
        };

        // serialize RemoteRDMADescriptor
        let mut machine_bytes = unsafe {
            serializer.clone_and_resize(machine_len).unwrap()
        };
        self.machine.serialize(&mut machine_bytes);
        true
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let mut deserializer = unsafe {
            bytes.clone()
        };

        // Deserialize regs
        let reg_len = core::mem::size_of::<RegDescriptor>();
        let reg_bytes = unsafe {
            deserializer.clone_and_resize(reg_len).unwrap()
        };
        let regs = RegDescriptor::deserialize(&reg_bytes)?;
        deserializer = unsafe {
            deserializer.truncate_header(reg_len).unwrap()
        };

        // Deserialize PageMap
        // 1. read the page table length as usize
        let pagemap_len = unsafe {
            *(deserializer.get_ptr() as *const usize)
        };
        deserializer = unsafe {
            deserializer.truncate_header(core::mem::size_of::<usize>()).unwrap()
        };
        // 2. deserialize the page table
        let page_map_bytes = unsafe {
            deserializer.clone_and_resize(pagemap_len).unwrap()
        };
        let page_map = PageMap::deserialize(&page_map_bytes)?;
        deserializer = unsafe {
            deserializer.truncate_header(pagemap_len).unwrap()
        };

        // Deserialize VMAs
        // 1. read the vma list length as usize
        let vma_len = unsafe {
            *(deserializer.get_ptr() as *const usize)
        };
        deserializer = unsafe {
            deserializer.truncate_header(core::mem::size_of::<usize>()).unwrap()
        };
        // 2. deserialize the vma list
        let vma = unsafe {
            core::slice::from_raw_parts(deserializer.get_ptr() as *const _ as *const VMADescriptor, vma_len/core::mem::size_of::<VMADescriptor>()).to_vec()
        };
        deserializer = unsafe {
            deserializer.truncate_header(vma_len).unwrap()
        };

        // Deserialize RemoteRDMADescriptor
        let machine_len = core::mem::size_of::<RemoteRDMADescriptor>();
        let machine_bytes = unsafe {
            deserializer.clone_and_resize(machine_len).unwrap()
        };
        let machine = RemoteRDMADescriptor::deserialize(&machine_bytes)?;
        Some(Self {
            regs: regs,
            page_table: page_map,
            vma: vma,
            machine: machine,
        })
    }

    fn serialization_len(&self) -> usize {
        self.regs.serialization_len()
        + core::mem::size_of::<usize>()
        + self.page_table.serialization_len()
        + core::mem::size_of::<usize>()
        + self.vma.len() * core::mem::size_of::<VMADescriptor>()
        + self.machine.serialization_len()
    }
}
