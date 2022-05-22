use alloc::vec;
use alloc::vec::Vec;
use KRdmaKit::rust_kernel_rdma_base::VmallocAllocator;
use rust_kernel_linux_util::LevelFilter::Debug;

use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;
use crate::*;

use os_network::bytes::BytesMut;
use os_network::serialize::Serialize;

use mitosis::descriptors::*;
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
        match cmd {
            0 => self.test_reg_descriptor(arg),
            1 => self.test_page_table(arg),
            3 => self.test_rdma_descriptor(arg),
            4 => self.test_mitosis_child_descriptor(arg),
            5 => self.test_vma_page_table(arg),
            6 => self.test_mitosis_parent_descriptor(arg),
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

        let mut res: RegDescriptor = Default::default();
        crate::log::debug!("sanity check regs fs {}, gs {}", reg.get_fs(), reg.get_gs());
        crate::log::debug!(
            "sanity check init regs fs {}, gs {}",
            res.get_fs(),
            res.get_gs()
        );

        res = RegDescriptor::deserialize(&bytes).unwrap();
        crate::log::debug!(
            "sanity check de-serialize regs fs {}, gs {}",
            res.get_fs(),
            res.get_gs()
        );

        assert_eq!(res.get_fs(), reg.get_fs());
        assert_eq!(res.get_gs(), reg.get_gs());
        assert_eq!(res, reg);

        crate::log::info!("pass RegDescriptor (de)serialization test\n");
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

        {
            // now deserialize
            let de_page_table: core::option::Option<FlatPageTable> =
                FlatPageTable::deserialize(&bytes);
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
        }

        log::info!("test page_table simple done");

        // make a complex test
        let task = Task::new();
        log::info!("check my task {:?}", task);

        let (_, pt) = task.generate_mm();
        log::debug!("Generated page table sz {}", pt.len());

        let mut memory = vec![0 as u8; pt.serialization_buf_len()];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };

        let result = pt.serialize(&mut bytes);
        if !result {
            log::error!("fail to serialize flat page table");
            return -1;
        }

        {
            // now deserialize
            let de_page_table: core::option::Option<FlatPageTable> =
                FlatPageTable::deserialize(&bytes);
            if de_page_table.is_none() {
                log::error!("failed to deserialize page table");
                return -1;
            }
            let de_page_table = de_page_table.unwrap();
            log::info!("Task de-serialize page table sz: {}", de_page_table.len());

            assert_eq!(de_page_table.len(), pt.len());
            assert_eq!(de_page_table, pt);
        }

        log::info!("test page_table task done\n");

        return 0;
    }

    /// Test the (de)serialization of RemoteRDMADescriptor
    #[inline(always)]
    fn test_rdma_descriptor(&self, _arg: c_ulong) -> c_long {
        let mut descriptor: RDMADescriptor = Default::default();
        descriptor.set_rkey(0xdeadbeaf).set_service_id(73);

        let mut memory = vec![0; core::mem::size_of::<RDMADescriptor>()];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };

        crate::log::debug!("pre-check rdma descriptor {:?}", descriptor);
        let result = descriptor.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize RDMADescriptor");
            return 0;
        }

        let result = RDMADescriptor::deserialize(&bytes);
        if result.is_none() {
            crate::log::error!("fail to deserialize RemoteRDMADescriptor");
            return 0;
        }

        crate::log::debug!("post-check rdma descriptor {:?}", descriptor);

        let result = result.unwrap();

        // check equality
        if result != descriptor {
            crate::log::error!("de-serialize fail on {:?}", result);
        }

        // make a complex tests
        let (_target, descriptor) =
            RDMADescriptor::new_from_dc_target_pool().expect("failed to get RDMA descriptor");
        log::debug!("sanity check RDMA descriptor content {:?}", descriptor);

        let result = descriptor.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize RDMADescriptor");
            return 0;
        }

        let result = RDMADescriptor::deserialize(&bytes);
        if result.is_none() {
            crate::log::error!("fail to deserialize RemoteRDMADescriptor");
            return 0;
        }

        let result = result.unwrap();
        if result != descriptor {
            crate::log::error!("de-serialize fail on {:?} for target content", result);
        }

        crate::log::info!("pass RDMADescriptor (de)serialization test\n");
        0
    }

    fn test_vma_page_table(&self, _arg: c_ulong) -> c_long {
        let mut pg_table = CompactPageTable::default();
        for i in 0..100 {
            pg_table.add_one(i, (i * 2) as _);
        }
        let mut memory = vec![0; pg_table.serialization_buf_len()];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };

        // serialize
        let result = pg_table.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize process descriptor");
            return 0;
        }
        // deserialize
        let result = CompactPageTable::deserialize(&bytes);
        if result.is_none() {
            crate::log::error!("fail to deserialize process descriptor");
            return 0;
        }

        if pg_table.table_len() != result.unwrap().table_len() {
            crate::log::error!("vector len mismatch");
        }
        0
    }

    fn test_mitosis_parent_descriptor(&self, _arg: c_ulong) -> c_long {
        crate::log::info!("test mitosis parent descriptor");
        let mut mac_info: RDMADescriptor = Default::default();
        mac_info.set_rkey(0xdeadbeaf).set_service_id(73);

        let task = Task::new();
        let (vma, _) = task.generate_mm();
        let mut pg_table = Vec::new_in(VmallocAllocator);
        for vm in vma.iter() {
            let mut vma_pg_table = CompactPageTable::default();
            {
                vma_pg_table.add_one((0x10 + vm.get_start()) as _, 4);
            }
            pg_table.push(vma_pg_table);
        }

        let descriptor = ParentDescriptor {
            regs: task.generate_reg_descriptor(),
            page_table: pg_table,
            vma,
            machine_info: mac_info.clone(),
        };

        log::debug!(
            "sanity check parent descriptor serialization sz: {}",
            descriptor.serialization_buf_len()
        );
        let mut memory = vec![0; descriptor.serialization_buf_len()];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };

        // serialize
        let result = descriptor.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize process descriptor");
            return 0;
        }

        // deserialize
        let result = ParentDescriptor::deserialize(&bytes);
        if result.is_none() {
            crate::log::error!("fail to deserialize process descriptor");
            return 0;
        }
        let result = result.unwrap();
        if result.page_table.len() != descriptor.page_table.len() {
            log::error!(
                "failed to deserialize page table, {}, {}",
                result.page_table.len(),
                descriptor.page_table.len(),
            );
        }
        log::debug!(
            "page table,des len {}, origin len {}",
            result.page_table.len(),
            descriptor.page_table.len(),
        );
        crate::log::debug!(
            "check mac_info {:?}\n {:?}",
            result.machine_info,
            descriptor.machine_info
        );
        
        crate::log::info!("pass process ParentDescriptor (de)serialization test\n");

        0
    }

    fn test_mitosis_child_descriptor(&self, _arg: c_ulong) -> c_long { 
       crate::log::info!("test mitosis parent descriptor");
        let mut mac_info: RDMADescriptor = Default::default();
        mac_info.set_rkey(0xdeadbeaf).set_service_id(73);

        let task = Task::new();
        let (vma, _) = task.generate_mm();
        let mut pg_table = Vec::new_in(VmallocAllocator);
        for vm in vma.iter() {
            let mut vma_pg_table = CompactPageTable::default();
            {
                vma_pg_table.add_one((0x10 + vm.get_start()) as _, 4);
            }
            pg_table.push(vma_pg_table);
        }

        let descriptor = ParentDescriptor {
            regs: task.generate_reg_descriptor(),
            page_table: pg_table,
            vma,
            machine_info: mac_info.clone(),
        };

        log::debug!(
            "sanity check parent descriptor serialization sz: {}",
            descriptor.serialization_buf_len()
        );
        let mut memory = vec![0; descriptor.serialization_buf_len()];
        let mut bytes = unsafe { BytesMut::from_raw(memory.as_mut_ptr(), memory.len()) };

        // serialize
        let result = descriptor.serialize(&mut bytes);
        if !result {
            crate::log::error!("fail to serialize process descriptor");
            return 0;
        }

        // deserialize
        crate::log::info!("pass process ChildDescriptor (de)serialization test\n");

        0
    }
}
