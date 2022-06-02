use crate::*;

use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;

pub(crate) struct MySyscallHandler;

use mitosis::descriptors::rdma::RDMADescriptor;
use mitosis::kern_wrappers::task::Task;
use mitosis::remote_paging::AccessInfo;
use mitosis::rust_kernel_linux_util::kthread;

use mitosis::os_network::{block_on_w_yield, timeout::TimeoutWRef};

pub const TIMEOUT_USEC: i64 = 10_000; // 10ms

// real system call implementations
impl MySyscallHandler {
    #[inline(always)]
    fn handle_prefetcher(&self, _arg: c_ulong) -> c_long {
        log::info!("start test prefetcher");

        // now test the prefetcher
        let factory = unsafe { mitosis::get_dc_factory_ref(0).unwrap() };

        // it is important to keep the _dc_target here
        // otherwise the remote end will fail to connect to the host w/ DCQP
        let (_dc_target, rdma_info) = RDMADescriptor::new_from_dc_target_pool().unwrap();
        let access_info = AccessInfo::new(&rdma_info).unwrap();

        let mut exe = DCAsyncPrefetcher::new(factory, access_info).unwrap();

        // 1. generate the page table
        let task = Task::new();
        let (_, pt) = task.generate_mm();

        let mut new_pt = Box::new(RemotePageTable::new());

        for (k, v) in pt.iter() {
            assert!(new_pt.map(VirtAddr::new(*k), PhysAddr::new(*v)).is_none());
        }

        let mut iter = RemotePageTableIter::new(&mut new_pt).unwrap();

        // randomly move the iterator
        for _ in 0..5 {
            if iter.next().is_none() {
                break;
            }
        }

        let iter_copy = unsafe { iter.clone() };

        // Add the current iterator to generate the requests
        exe.execute_reqs(iter, StepPrefetcher::<PageEntry, 2>::new());

        log::info!("pending reqs: {}", exe.num_pending());

        // ensure we are able to poll the request
        kthread::sleep(1);
        let mut timeout_prefetcher = TimeoutWRef::new(&mut exe, TIMEOUT_USEC);

        // pop the first request
        let res = block_on_w_yield(&mut timeout_prefetcher);
        log::info!("polled res {:?}", res);
        assert!(res.is_ok());
        // free the page
        unsafe { mitosis::bindings::pmem_free_page(res.unwrap()) };

        // pop the second request
        let res = block_on_w_yield(&mut timeout_prefetcher);
        log::info!("polled res {:?}", res);
        assert!(res.is_ok());
        // free the page
        unsafe { mitosis::bindings::pmem_free_page(res.unwrap()) };

        log::info!("pending reqs: {}", exe.num_pending());
        assert_eq!(exe.num_pending(), 0);

        // finally, we check the prefetched page cannot be prefetch again
        exe.execute_reqs(iter_copy, ConstPrefetcher::<PageEntry, 2>::new());
        log::info!("pending reqs after the prefetch: {}", exe.num_pending());
        assert_eq!(exe.num_pending(), 0);

        self.test_page_table_self_cloning();
        0
    }
}

impl MySyscallHandler {
    fn test_page_table_self_cloning(&self) {
        log::info!("===============start test page table self cloning===============");
        let mut page_table = RemotePageTable::new();

        for i in 0..20 {
            let u = PhysAddr::encode((i + 1) * 4096, PhysAddrBitFlag::Cache as _);
            page_table.map(VirtAddr::new(i * 4096), PhysAddr::new(u));
        }

        log::info!("get initial page size:{}", page_table.len());
        page_table.print_self();

        let mut copied_pg_table = page_table.copy();
        log::info!("get initial page size:{}", copied_pg_table.len());
        copied_pg_table.print_self();

        log::info!("===============end test page table self cloning===============");
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
    #[allow(unreachable_patterns)]
    #[inline]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        crate::log::debug!("in ioctrl");
        match cmd {
            _CALL_NIL => self.handle_prefetcher(arg),
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
