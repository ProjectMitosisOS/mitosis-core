use alloc::vec::Vec;

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

        let mut keys = Vec::new();

        // test the basic transalate works
        for (k, v) in pt.iter() {
            let nv = new_pt.translate(VirtAddr::new(*k));
            assert!(nv.is_some());
            assert_eq!(nv.unwrap().as_u64(), *v);
            keys.push(k);
        }

        crate::log::debug!("check page table entry counts: {}", counter);

        // now check the iterators
        crate::log::info!("Now check the iterators");

        let iter = unsafe { RemotePageTableIter::new(&mut new_pt).unwrap() };
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

        // test partial iterators
        //let new_counter = counter2 / 2;
        let new_counter = 0;
        counter = 0;

        let mut temp: (*mut PageTable, usize) = (core::ptr::null_mut(), 0);

        // note that the keys are not sorted
        for (k, _) in pt.iter() {
            if counter == new_counter {
                crate::log::debug!("peek counter: {}", counter);
                temp = new_pt.find_l1_page_idx(VirtAddr::new(*k)).unwrap();
                break;
            }
            counter += 1;
        }

        let iter1 = unsafe { RemotePageTableIter::new_from_l1(temp.0, temp.1) };

        let mut counter3 = 0;
        for _addr in iter1 {
            // crate::log::info!("check addr {:?}", addr);
            counter3 += 1;
        }

        crate::log::debug!(
            "check partial index temp {:?}, iter num: {}",
            temp,
            counter3
        );

        // finally, we test the case when the keys are sorted
        keys.sort();

        temp = new_pt
            .find_l1_page_idx(VirtAddr::new(*(keys[keys.len() / 2])))
            .unwrap();

        let iter1 = unsafe { RemotePageTableIter::new_from_l1(temp.0, temp.1) };

        let mut counter3 = 0;
        for _addr in iter1 {
            // crate::log::info!("check addr {:?}", addr);
            counter3 += 1;
        }
        crate::log::info!(
            "The counter3 ({}) should be roughly half of {}",
            counter3,
            keys.len()
        );
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
