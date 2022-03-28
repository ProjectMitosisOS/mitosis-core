use alloc::vec::Vec;

use os_network::bytes::BytesMut;

use crate::linux_kernel_module;

// sub descriptors 
pub mod reg;
pub use reg::RegDescriptor;

pub mod page_table;
pub use page_table::FlatPageTable;

pub mod rdma;
pub use rdma::RDMADescriptor;

pub mod vma; 
pub use vma::VMADescriptor;

/// The kernel-space descriptor of MITOSIS
#[allow(dead_code)]
#[derive(Default)]
pub struct Descriptor {
    pub regs: RegDescriptor,
    pub page_table: FlatPageTable,
    pub vma: Vec<VMADescriptor>,
    pub machine_info: RDMADescriptor,
}

impl os_network::serialize::Serialize for Descriptor {
    /// Serialization format:
    /// ```
    /// | RegDescriptor <-sizeof(RegDescriptor)-> | PageMap
    /// | VMA descriptor length in bytes <-8 bytes-> | VMA descriptor | RDMADescriptor |
    /// ```
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        if bytes.len() < self.serialization_buf_len() {
            crate::log::error!("failed to serialize: buffer space not enough");
            return false;
        }

        let mut serializer = unsafe { bytes.clone() };

        let reg_len = self.regs.serialization_buf_len();
        let pagemap_len = self.page_table.serialization_buf_len();
        let vma_len = self.vma.len() * core::mem::size_of::<VMADescriptor>();
        let machine_len = self.machine_info.serialization_buf_len();

        // serialize regs
        let mut reg_bytes = unsafe { serializer.clone_and_resize(reg_len).unwrap() };
        self.regs.serialize(&mut reg_bytes);
        serializer = unsafe { serializer.truncate_header(reg_len).unwrap() };

        // serialize PageMap length in bytes (usize)
        unsafe {
            *(serializer.get_ptr() as *mut usize) = pagemap_len;
        }
        serializer = unsafe {
            serializer
                .truncate_header(core::mem::size_of::<usize>())
                .unwrap()
        };

        // serialize PageMap
        let mut page_map_bytes = unsafe { serializer.clone_and_resize(pagemap_len).unwrap() };
        self.page_table.serialize(&mut page_map_bytes);
        serializer = unsafe { serializer.truncate_header(pagemap_len).unwrap() };

        // serialize VMADescriptors' length in bytes (usize)
        unsafe {
            *(serializer.get_ptr() as *mut usize) = vma_len;
        }
        serializer = unsafe {
            serializer
                .truncate_header(core::mem::size_of::<usize>())
                .unwrap()
        };

        // serialize VMADescriptors
        let mut vma_descriptor_bytes = unsafe { serializer.clone_and_resize(vma_len).unwrap() };
        let raw_vm_descriptors =
            unsafe { BytesMut::from_raw(self.vma.as_ptr() as *mut u8, vma_len) };
        vma_descriptor_bytes.copy(&raw_vm_descriptors, 0);
        serializer = unsafe { serializer.truncate_header(vma_len).unwrap() };

        // serialize RemoteRDMADescriptor
        let mut machine_bytes = unsafe { serializer.clone_and_resize(machine_len).unwrap() };
        self.machine_info.serialize(&mut machine_bytes);
        true
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let mut deserializer = unsafe { bytes.clone() };

        // Deserialize regs
        let reg_len = core::mem::size_of::<RegDescriptor>();
        let reg_bytes = unsafe { deserializer.clone_and_resize(reg_len) }?;
        let regs = RegDescriptor::deserialize(&reg_bytes)?;
        deserializer = unsafe { deserializer.truncate_header(reg_len) }?;

        // Deserialize PageMap
        // 1. read the page table length as usize
        let pagemap_len = unsafe { *(deserializer.get_ptr() as *const usize) };
        deserializer = unsafe { deserializer.truncate_header(core::mem::size_of::<usize>()) }?;
        // 2. deserialize the page table
        let page_map_bytes = unsafe { deserializer.clone_and_resize(pagemap_len) }?;
        let page_map = FlatPageTable::deserialize(&page_map_bytes)?;
        deserializer = unsafe { deserializer.truncate_header(pagemap_len) }?;

        // Deserialize VMAs
        // 1. read the vma list length as usize
        let vma_len = unsafe { *(deserializer.get_ptr() as *const usize) };
        deserializer = unsafe { deserializer.truncate_header(core::mem::size_of::<usize>()) }?;
        // 2. deserialize the vma list
        let vma = unsafe {
            core::slice::from_raw_parts(
                deserializer.get_ptr() as *const _ as *const VMADescriptor,
                vma_len / core::mem::size_of::<VMADescriptor>(),
            )
            .to_vec()
        };
        deserializer = unsafe { deserializer.truncate_header(vma_len) }?;

        // Deserialize RemoteRDMADescriptor
        let machine_len = core::mem::size_of::<RDMADescriptor>();
        let machine_bytes = unsafe { deserializer.clone_and_resize(machine_len) }?;
        let machine = RDMADescriptor::deserialize(&machine_bytes)?;

        Some(Self {
            regs: regs,
            page_table: page_map,
            vma: vma,
            machine_info: machine,
        })
    }

    fn serialization_buf_len(&self) -> usize {
        self.regs.serialization_buf_len()
            + self.page_table.serialization_buf_len()
            + core::mem::size_of::<usize>() // #VMA descriptors 
            + self.vma.len() * core::mem::size_of::<VMADescriptor>()
            + self.machine_info.serialization_buf_len()
    }
}
