use crate::*;

use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;

pub(crate) struct MySyscallHandler;

use mitosis::kern_wrappers::task::Task;

// real system call implementations
impl MySyscallHandler {
    #[inline(always)]
    fn handle_page_mapping_test(&self, arg: c_ulong) -> c_long {
        crate::log::debug!("handle page mapping, with arg {}", arg);

        // initialize the page table as the caller tasks's pagetable
        let task = Task::new();
        let (_, pt) = task.generate_mm();

        let mut new_pt = Box::new(RemotePageTable::new());

        let mut counter = 0;
        for (k, v) in pt.iter() {
            let phy_addr = PhysAddr::try_new(*v);
            if phy_addr.is_err() {
                crate::log::error!(
                    "Failed to create physical addr: {:?}, with virtual addr: 0x{:x}",
                    phy_addr,
                    *v
                );
                pt.translate(*k)
                    .map(|v| crate::log::debug!("Double-check the phys: {:x}", v));
                continue;
            }
            assert!(new_pt.map(VirtAddr::new(*k), PhysAddr::new(*v)).is_none());
            counter += 1;
        }

        // test the basic transalate works
        for (k, v) in pt.iter() {
            let nv = new_pt.translate(VirtAddr::new(*k));
            assert!(nv.is_some());
            assert_eq!(nv.unwrap().as_u64(), *v);
        }

        crate::log::debug!("check page table entry counts: {}", counter);

        // now check the iterators
        crate::log::info!("Now check the iterators");

        let iter = RemotePageTableIter::new(&mut new_pt).unwrap();
        crate::log::debug!("sanity check iter {:?}", iter);

        let mut counter2 = 0;
        for _addr in iter {
            // crate::log::info!("check addr {:?}", addr);
            counter2 += 1;
        }

        crate::log::info!(
            "sanity check iterator counter: {}, previous: {} should match",
            counter2,
            counter
        );
        assert_eq!(counter, counter2);

        0
    }
}

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
            _CALL_NIL => self.handle_page_mapping_test(arg),
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
