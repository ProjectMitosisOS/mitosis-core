use crate::linux_kernel_module;

use hashbrown::HashMap;

use os_network::bytes::BytesMut;

use super::{AddrType, RemotePage};

/// Record the mapping between the va and remote pa of a process
#[derive(Default)]
pub struct PageMap(pub HashMap<AddrType, RemotePage>);

impl os_network::serialize::Serialize for PageMap {
    /// Serialization format:
    /// ```
    /// | AddrType <-8 bytes-> | Remote Page <-16 bytes-> | AddrType <-8 bytes-> | Remote Page <-16 bytes-> |
    /// ```
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        if bytes.len() < self.serialization_buf_len() {
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

    fn serialization_buf_len(&self) -> usize {
        let count = self.0.len();
        let entry_size = core::mem::size_of::<AddrType>() + core::mem::size_of::<RemotePage>();
        entry_size * count
    }
}
