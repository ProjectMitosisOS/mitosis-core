use alloc::vec;

use crate::*;
use crate::linux_kernel_module::c_types::*;
use crate::linux_kernel_module::bindings::vm_area_struct;

use os_network::bytes::BytesMut;
use os_network::serialize::Serialize;

use mitosis::descriptor::PageMap;
use mitosis::descriptor::Descriptor;
use mitosis::descriptor::RemotePage;
use mitosis::descriptor::VMADescriptor;
use mitosis::descriptor::reg::RegDescriptor;
use mitosis::descriptor::RemoteRDMADescriptor;

pub(crate) struct MySyscallHandler;

// FIXME: we need to place these with auto-generated code, e.g., proc_macros
// But currently, we don't have time to do so
#[allow(non_upper_case_globals)]
impl FileOperations for MySyscallHandler
{
    #[inline]
    fn open(
        _file: *mut crate::linux_kernel_module::bindings::file,
    ) -> crate::linux_kernel_module::KernelResult<Self> {
        Ok(Self)
    }

    #[allow(non_snake_case)]
    #[inline]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        crate::log::debug!("in ioctrl");
        match cmd {
            0 => self.test_reg_descriptor(arg),
            1 => self.test_page_map(arg),
            3 => self.test_remote_rdma_descriptor(arg),
            4 => self.test_mitosis_descriptor(arg),
            _ => {
                crate::log::error!("unknown system call command ID {}", cmd);
                -1
            }
        }
    }

    #[inline]
    fn mmap(&mut self, _vma_p: *mut vm_area_struct) -> c_int {
        unimplemented!();
    }
}

impl MySyscallHandler {
    /// Test the (de)serialization of RegDescriptor
    #[inline(always)]
    fn test_reg_descriptor(&self, _arg: c_ulong) -> c_long {
        let mut reg: RegDescriptor = Default::default();
        reg.others.r15 = 0x12345678;
        reg.fs = 0x87654321;
        let mut memory = vec![0 as u8; core::mem::size_of::<RegDescriptor>()];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };
        let result = reg.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize reg");
            return 0;
        }
        let result = RegDescriptor::deserialize(&bytes).unwrap();
        if result.others.r15 != reg.others.r15 {
            crate::log::error!("r15: expected: 0x{:x}, got: 0x{:x}", reg.others.r15, result.others.r15);
            return 0;
        }
        if result.fs != reg.fs {
            crate::log::error!("fs: expected: 0x{:x}, got 0x{:x}", reg.fs, result.fs);
            return 0;
        }
        crate::log::info!("pass RegDescriptor (de)serialization test");
        0
    }

    /// Test the (de)serialization of PageMap
    #[inline(always)]
    fn test_page_map(&self, _arg: c_ulong) -> c_long {
        let mut page_map: PageMap = Default::default();
        let mut remote_page = RemotePage::default();
        remote_page.addr = 0xdeadbeaf;
        remote_page.dct_key = 0xdeaddead;
        page_map.0.insert(0x1, remote_page);
        remote_page.dct_key = 0xbeafbeaf;
        page_map.0.insert(0x2, remote_page);
        let size = 2 * (core::mem::size_of::<u64>() + core::mem::size_of::<RemotePage>());
        let mut memory = vec![0 as u8; size];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };
        let result = page_map.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize page_map");
            return 0;
        }
        let result = PageMap::deserialize(&mut bytes);
        if result.is_none() {
            crate::log::error!("fail to deserialize");
            return 0;
        }
        
        let result = result.unwrap();
        if result.0.get(&0x1).is_none() {
            crate::log::error!("fail to find key 0x12345678");
            return 0;
        }
        if result.0.get(&0x2).is_none() {
            crate::log::error!("fail to find key 0x87654321");
            return 0;
        }
        if result.0.get(&0x2).unwrap().dct_key != 0xbeafbeaf {
            crate::log::error!("expected: 0x{:x}, got: 0x{:x}", 0xbeafbeaf as u32, result.0.get(&0x2).unwrap().dct_key);
            return 0;
        }
        crate::log::info!("pass PageMap (de)serialization test");
        0
    }

    /// Test the (de)serialization of RemoteRDMADescriptor
    #[inline(always)]
    fn test_remote_rdma_descriptor(&self, _arg: c_ulong) -> c_long {
        let mut descriptor: RemoteRDMADescriptor = Default::default();
        descriptor.rkey = 0xdeadbeaf;
        let mut memory = vec![0; core::mem::size_of::<RemoteRDMADescriptor>()];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };
        
        let result = descriptor.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize RemoteRDMADescriptor");
            return 0;
        }
        let result = RemoteRDMADescriptor::deserialize(&bytes);
        if result.is_none() {
            crate::log::error!("fail to deserialize RemoteRDMADescriptor");
            return 0;
        }

        let result = result.unwrap();
        if result.rkey != descriptor.rkey {
            crate::log::error!("expected: 0x{:x}, got: 0x{:x}", descriptor.rkey, result.rkey);
            return 0;
        }
        crate::log::info!("pass RemoteRDMADescriptor (de)serialization test");
        0
    }

    /// Test the (de)serialization of mitosis Descriptor
    #[inline(always)]
    fn test_mitosis_descriptor(&self, _arg: c_ulong) -> c_long {
        let mut descriptor: Descriptor = Default::default();
        descriptor.page_table.0.insert(0x1, RemotePage::default());
        descriptor.page_table.0.insert(0x2, RemotePage::default());
        descriptor.vma.push(VMADescriptor::default());

        let size = descriptor.serialization_len();
        let mut memory = vec![0; size];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };

        let result = descriptor.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize mitosis descriptor");
            return 0;
        }

        let result = Descriptor::deserialize(&bytes);
        if result.is_none() {
            crate::log::error!("fail to deserialize mitosis descriptor");
            return 0;
        }

        let result = result.unwrap();
        if result.page_table.0.len() != 2 {
            crate::log::error!("expected: {}, got: {}", 2, result.page_table.0.len());
            return 0;
        }
        if result.vma.len() != 1 {
            crate::log::error!("expected: {}, got: {}", 1, result.vma.len());
            return 0;
        }
        crate::log::info!("pass mitosis descriptor (de)serialization test");
        0
    }
}
