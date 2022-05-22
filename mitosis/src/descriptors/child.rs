use crate::linux_kernel_module;
use alloc::vec::Vec;
use os_network::bytes::BytesMut;

use super::rdma::RDMADescriptor;
use super::reg::RegDescriptor;
use super::vma::VMADescriptor;

use super::page_table::FlatPageTable;

use crate::kern_wrappers::mm::{PhyAddrType, VirtAddrType};
use crate::kern_wrappers::task::Task;
use crate::remote_paging::AccessInfo;

/// The kernel-space process descriptor of MITOSIS
/// The descriptors should be generate by the task
#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct ChildDescriptor {
    pub regs: RegDescriptor,

    pub page_table: FlatPageTable,

    pub vma: Vec<VMADescriptor>,
    pub machine_info: RDMADescriptor,
}

impl ChildDescriptor {
    pub fn new(task: &crate::kern_wrappers::task::Task, mac_info: RDMADescriptor) -> Self {
        let (vma, pt) = task.generate_mm();
        Self {
            regs: task.generate_reg_descriptor(),
            page_table: pt,
            vma: vma,
            machine_info: mac_info,
        }
    }

    #[inline]
    pub fn lookup_pg_table(&self, virt: VirtAddrType) -> Option<PhyAddrType> {
        self.page_table.get(virt)
    }

    /// Apply the descriptor into current process
    #[inline]
    pub fn apply_to(&self, file: *mut crate::bindings::file) {
        let mut task = Task::new();
        // 1. Unmap origin vma regions
        task.unmap_self();
        let access_info = AccessInfo::new(&self.machine_info).unwrap();

        // 2. Map new vma regions
        (&self.vma).into_iter().for_each(|m| {
            let vma = unsafe { task.map_one_region(file, m) };

            #[allow(dead_code)]
            let vma = vma.unwrap();

            // tune the bits
            let origin_vma_flags =
                unsafe { crate::bindings::VMFlags::from_bits_unchecked(m.flags) };
            // crate::log::info!("orign vma: {:?}", origin_vma_flags);
            if origin_vma_flags.contains(crate::bindings::VMFlags::VM_ALLOC) {
                // set the vma
                crate::kern_wrappers::vma::VMA::new(vma).set_alloc();
            }

            if cfg!(feature = "eager-resume") {
                let (size, start) = (m.get_sz(), m.get_start());
                for addr in (start..start + size).step_by(4096) {
                    if let Some(new_page_p) = unsafe { self.read_remote_page(addr, &access_info) } {
                        // FIXME: 52 is hard-coded
                        vma.vm_page_prot.pgprot =
                            vma.vm_page_prot.pgprot | (((1 as u64) << 52) as u64); // present bit
                        let _ =
                            unsafe { crate::bindings::pmem_vm_insert_page(vma, addr, new_page_p) };
                    }
                }
            }
        });
        // 3. Re-set states
        task.set_mm_reg_states(&self.regs);
    }
}

impl ChildDescriptor {
    /// Resume one page at remote side
    ///
    /// @param remote_va: remote virt-addr
    /// @param access_info: remote network meta info
    #[inline]
    pub unsafe fn read_remote_page(
        &self,
        remote_va: VirtAddrType,
        access_info: &AccessInfo,
    ) -> Option<*mut crate::bindings::page> {
        let remote_pa = self.lookup_pg_table(remote_va);
        if remote_pa.is_none() {
            return None;
        }
        let new_page_p = crate::bindings::pmem_alloc_page(crate::bindings::PMEM_GFP_HIGHUSER);
        let new_page_pa = crate::bindings::pmem_page_to_phy(new_page_p) as u64;
        let res = crate::remote_paging::RemotePagingService::remote_read(
            new_page_pa,
            remote_pa.unwrap(),
            4096,
            access_info,
        );
        return match res {
            Ok(_) => Some(new_page_p),
            Err(e) => {
                crate::log::error!("Failed to read the remote page {:?}", e);
                None
            }
        };
    }
}

impl os_network::serialize::Serialize for ChildDescriptor {
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        // Note, since we currently don't support multi-fork, so child serialize is not implemented
        unimplemented!();
    }

    /// De-serialize from a message buffer
    /// **Warning** 
    /// - The buffer to be serialized must be generated from the ParentDescriptor. 
    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {

        /* 
        let mut cur = unsafe { bytes.truncate_header(0).unwrap() };

        // regs
        let regs = RegDescriptor::deserialize(&cur)?;
        cur = unsafe { cur.truncate_header(regs.serialization_buf_len())? };

        let mut page_table = FlatPageTable::new();

        let mut pt = Vec::new_in(VmallocAllocator);
        // VMA page table count
        let mut count: usize = 0;
        let off = unsafe { cur.memcpy_deserialize(&mut count)? };
        cur = unsafe { cur.truncate_header(off)? };        

        for _ in 0..count {
            let vma_pg_table = CompactPageTable::deserialize(&cur)?;
            cur = unsafe { cur.truncate_header(vma_pg_table.serialization_buf_len())? };
            pt.push(vma_pg_table);
        }

        // vmas
        let mut vmas = Vec::new();
        let mut count: usize = 0;
        let off = unsafe { cur.memcpy_deserialize(&mut count)? };
        cur = unsafe { cur.truncate_header(off)? };

        for _ in 0..count {
            let vma = VMADescriptor::deserialize(&cur)?;
            cur = unsafe { cur.truncate_header(vma.serialization_buf_len())? };
            vmas.push(vma);
        }
        let machine_info = RDMADescriptor::deserialize(&cur)?;        
        */
        
        unimplemented!();
        /* 
        let cur = unsafe { bytes.truncate_header(0).unwrap() };

        // regs
        let regs = RegDescriptor::deserialize(&cur)?;
        let cur = unsafe { cur.truncate_header(regs.serialization_buf_len())? };

        // page table
        let pt = FlatPageTable::deserialize(&cur)?;
        let cur = unsafe { cur.truncate_header(pt.serialization_buf_len())? };

        // vmas
        let mut vmas = Vec::new();
        let mut count: usize = 0;
        let off = unsafe { cur.memcpy_deserialize(&mut count)? };
        let mut cur = unsafe { cur.truncate_header(off)? };

        for _ in 0..count {
            let vma = VMADescriptor::deserialize(&cur)?;
            cur = unsafe { cur.truncate_header(vma.serialization_buf_len())? };
            vmas.push(vma);
        }

        // machine info
        let machine_info = RDMADescriptor::deserialize(&cur)?;

        Some(Self {
            regs: regs,
            page_table: pt,
            vma: vmas,
            machine_info: machine_info,
        })*/
    }

    fn serialization_buf_len(&self) -> usize {
        self.regs.serialization_buf_len()
            + self.page_table.serialization_buf_len()
            + core::mem::size_of::<usize>() // the number of VMA descriptors 
            + self.vma.len() * core::mem::size_of::<VMADescriptor>()
            + self.machine_info.serialization_buf_len()
    }
}
