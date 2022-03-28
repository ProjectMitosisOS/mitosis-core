use alloc::vec;

use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;
use crate::*;

use os_network::bytes::BytesMut;
use os_network::serialize::Serialize;

use mitosis::descriptor::*;
use mitosis::kern_wrappers::task::Task;

pub(crate) struct MySyscallHandler;

// FIXME: we need to place these with auto-generated code, e.g., proc_macros
// But currently, we don't have time to do so
#[allow(non_upper_case_globals)]
impl FileOperations for MySyscallHandler {
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
            1 => self.test_page_table(arg),
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

        let reg: RegDescriptor = Task::new().generate_reg_descriptor(); 

        let mut memory = vec![0 as u8; core::mem::size_of::<RegDescriptor>()];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };
        let result = reg.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize reg");
            return 0;
        }

        let mut res : RegDescriptor = Default::default(); 
        crate::log::debug!("sanity check regs fs {}, gs {}",reg.get_fs(), reg.get_gs());
        crate::log::debug!("sanity check init regs fs {}, gs {}",res.get_fs(), res.get_gs());

        res = RegDescriptor::deserialize(&bytes).unwrap();
        crate::log::debug!("sanity check de-serialize regs fs {}, gs {}",res.get_fs(), res.get_gs());

        assert_eq!(res.get_fs(), reg.get_fs());
        assert_eq!(res.get_gs(), reg.get_gs());
        
        crate::log::info!("pass RegDescriptor (de)serialization test");
        0
    }

    /// Test the (de)serialization of PageMap
    #[inline(always)]
    fn test_page_table(&self, _arg: c_ulong) -> c_long {
        let mut page_table = FlatPageTable::new();
        page_table.add_one(0xdeadbeaf, 73).add_one(0xffff, 64);

        let mut memory = vec![0 as u8; page_table.serialization_buf_len()];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };

        let result = page_table.serialize(&mut bytes);
        if !result {
            log::error!("fail to serialize flat page table");
            return -1;
        }

        crate::log::debug!("{:?}", bytes);

        // now deserialize
        let de_page_table: core::option::Option<FlatPageTable> = FlatPageTable::deserialize(&bytes);
        if de_page_table.is_none() {
            log::error!("failed to deserialize page table");
            return -1;
        }
        let de_page_table = de_page_table.unwrap();
        log::debug!("de page table {:?}", de_page_table);

        assert_eq!(de_page_table.len(), page_table.len());
        assert_eq!(
            de_page_table.get(0xdeadbeaf).unwrap(),
            page_table.get(0xdeadbeaf).unwrap()
        );
        assert_eq!(
            de_page_table.get(0xffff).unwrap(),
            page_table.get(0xffff).unwrap()
        );

        log::info!("test page_table done");

        return 0;
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
            crate::log::error!(
                "expected: 0x{:x}, got: 0x{:x}",
                descriptor.rkey,
                result.rkey
            );
            return 0;
        }
        crate::log::info!("pass RemoteRDMADescriptor (de)serialization test");
        0
    }

    /// Test the (de)serialization of mitosis Descriptor
    #[inline(always)]
    fn test_mitosis_descriptor(&self, _arg: c_ulong) -> c_long {
        /*
        let mut descriptor: Descriptor = Default::default();
        descriptor.page_table.0.insert(0x1, RemotePage::default());
        descriptor.page_table.0.insert(0x2, RemotePage::default());
        descriptor.vma.push(VMADescriptor::default());

        let size = descriptor.serialization_buf_len();
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
        */
        0
    }
}
