use core::alloc::Layout;
use core::ptr::NonNull;

use crate::{
    kern_wrappers::mm::{PhyAddrType, VirtAddrType},
    linux_kernel_module,
};

use hashbrown::{hash_map::DefaultHashBuilder, HashMap};

use os_network::bytes::BytesMut;

#[derive(Clone, Default)]
pub struct PageMapAllocator;

unsafe impl hashbrown::raw::Allocator for PageMapAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<u8>, ()> {
        match layout.size() {
            0 => Ok(layout.dangling()),
            // SAFETY: `layout` is non-zero in size,
            size => {
                let raw_ptr = unsafe { crate::bindings::vmalloc(size as u64) } as *mut u8;
                let ptr = NonNull::new(raw_ptr).expect("vmalloc should not fail");

                // crate::log::debug!("check allocate: {:?}, ptr: {:?}", layout,ptr);
                Ok(ptr)
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        crate::bindings::vfree(ptr.as_ptr() as *const linux_kernel_module::c_types::c_void);
    }
}

type PageMap = HashMap<VirtAddrType, PhyAddrType, DefaultHashBuilder, PageMapAllocator>;

/// Record the mapping between the va and remote pa of a process
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct FlatPageTable(pub PageMap);

impl FlatPageTable {
    pub fn new() -> Self {
        let res: PageMap = Default::default();
        Self(res)
    }

    pub fn add_one(&mut self, v: VirtAddrType, p: PhyAddrType) -> &mut Self {
        self.0.insert(v, p);
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn translate(&self, vaddr: VirtAddrType) -> core::option::Option<PhyAddrType> {
        self.0.get(&vaddr).map(|v| *v)
    }

    pub fn iter(&self) -> hashbrown::hash_map::Iter<'_, VirtAddrType, PhyAddrType> {
        self.0.iter()
    }

    pub fn calculate_serialization_buf_len(&self, len: usize) -> usize {
        (core::mem::size_of::<VirtAddrType>() + core::mem::size_of::<PhyAddrType>()) * len
            + core::mem::size_of_val(&self.0.len())
    }
}

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
        let mut cur = unsafe { cur.truncate_header(sz).unwrap() };

        for (vaddr, paddr) in self.0.iter() {
            let sz0 = unsafe { cur.memcpy_serialize_at(0, vaddr).unwrap() };
            let sz1 = unsafe { cur.memcpy_serialize_at(sz0, paddr).unwrap() };
            cur = unsafe { cur.truncate_header(sz0 + sz1).unwrap() };
        }
        return true;
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let mut res: PageMap = Default::default();
        let mut count: usize = 0;
        let off = unsafe { bytes.memcpy_deserialize(&mut count)? };

        let mut cur = unsafe { bytes.truncate_header(off)? };

        for _ in 0..count {
            let mut virt: VirtAddrType = 0;
            let mut phy: PhyAddrType = 0;

            let sz0 = unsafe { cur.memcpy_deserialize_at(0, &mut virt)? };
            let sz1 = unsafe { cur.memcpy_deserialize_at(sz0, &mut phy)? };

            // crate::log::debug!("de serialize {:x}, {}", virt, phy);

            // TODO: we need to identify that it is remote
            res.insert(virt, phy);

            cur = unsafe { cur.truncate_header(sz0 + sz1)? };
        }
        // crate::log::debug!("pre-check {:?}", res);

        Some(FlatPageTable(res))
    }

    fn serialization_buf_len(&self) -> usize {
        self.calculate_serialization_buf_len(self.len())
    }
}
