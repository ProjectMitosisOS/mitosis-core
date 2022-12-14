use crate::*;

use crate::linux_kernel_module::bindings::vm_area_struct;
use crate::linux_kernel_module::c_types::*;

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
            0 => self.handle_basic(arg),
            1 => self.handle_page_test(arg),
            3 => self.handle_page_table_test(arg),
            4 => self.handle_basic_2(arg),
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

use mitosis::kern_wrappers::*;
use mitosis::shadow_process::*;

// real system call implementations
impl MySyscallHandler {
    #[inline(always)]
    fn handle_basic(&self, arg: c_ulong) -> c_long {
        crate::log::debug!("handle basic tests, with arg {}", arg);

        let task = task::Task::new();
        let mm = task.get_memory_descriptor();

        let mut vma_cnt = 0;
        let mut vma_file_cnt = 0;

        for vma in mm.get_vma_iter() {
            vma_cnt += 1;
            // we must not create the shadow process for COW
            // otherwise, the kernel module will be blocked
            let s_vma = ShadowVMA::new(vma, false);
            if s_vma.backed_by_file() {
                vma_file_cnt += 1;
            }
        }
        log::debug!("vma cnt {}, backed by file cnt {}", vma_cnt, vma_file_cnt);

        0
    }

    // The same version as handle_basic, with COW 
    #[inline(always)]
    fn handle_basic_2(&self, _arg: c_ulong) -> c_long {
        // TODO: COW seems cannot work with the unittest framework
        /*
        crate::log::debug!("handle basic tests, with arg {}", arg);

        let task = task::Task::new();
        let mm = task.get_memory_descriptor();

        let mut vma_cnt = 0;
        let mut vma_file_cnt = 0;

        for vma in mm.get_vma_iter() {
            vma_cnt += 1;
            // we must not create the shadow process for COW
            // otherwise, the kernel module will be blocked
            let s_vma = ShadowVMA::new(vma, true);
            if s_vma.backed_by_file() {
                vma_file_cnt += 1;
            }
        }
        log::debug!("cow vma cnt {}, backed by file cnt {}", vma_cnt, vma_file_cnt);
        */
        0
    }    

    #[inline(always)]
    fn handle_page_test(&self, arg: c_ulong) -> c_long {
        crate::log::debug!("handle page tests, with arg {}", arg);
        let page = unsafe { Copy4KPage::new(arg as _).expect("failed to create the page") };

        // now check the content
        let bytes = unsafe { page.to_bytes().clone_and_resize(16).unwrap() };
        log::debug!("check bytes content: {:?}", bytes);

        let mut pg = ShadowPageTable::<Copy4KPage>::new();
        pg.add_page(page);
        0
    }

    #[inline(always)]
    fn handle_page_table_test(&self, _arg: c_ulong) -> c_long {
        log::debug!("start test handle page table"); 
        let mut mac_info: mitosis::descriptors::RDMADescriptor = Default::default();
        mac_info.set_rkey(0xdeadbeaf).set_service_id(73);

        let _sp = ShadowProcess::new_copy(mac_info.clone());
        log::debug!("page table test done");
        0
    }
}
