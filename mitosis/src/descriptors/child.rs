use crate::linux_kernel_module;
use alloc::vec::Vec;
use os_network::bytes::BytesMut;

use super::rdma::RDMADescriptor;
use super::reg::RegDescriptor;
use super::vma::VMADescriptor;

#[cfg(not(feature = "prefetch"))]
use super::page_table::FlatPageTable;

#[cfg(feature = "prefetch")]
use crate::remote_mapping::{PhysAddr, RemotePageTable, RemotePageTableIter, VirtAddr};

#[allow(unused_imports)]
use super::parent::{CompactPageTable, Offset, Value};

use crate::kern_wrappers::mm::{PhyAddrType, VirtAddrType};
use crate::kern_wrappers::task::Task;
use crate::remote_paging::AccessInfo;

/// The kernel-space process descriptor of MITOSIS
/// The descriptors should be generate by the task
#[allow(dead_code)]
#[derive(Default)]
pub struct ChildDescriptor {
    pub regs: RegDescriptor,

    #[cfg(not(feature = "prefetch"))]
    pub page_table: FlatPageTable,

    #[cfg(feature = "prefetch")]
    pub page_table: RemotePageTable,

    pub vma: Vec<VMADescriptor>,
    pub machine_info: RDMADescriptor,
}

impl ChildDescriptor {
    /// # Warning: Deperacted
    /// The ChildDescriptor can only be built from the ParentDescriptor's
    /// serialzied buffer
    pub fn new(_task: &crate::kern_wrappers::task::Task, _mac_info: RDMADescriptor) -> Self {
        unimplemented!();
        /*
        let (vma, pt) = task.generate_mm();
        Self {
            regs: task.generate_reg_descriptor(),
            page_table: pt,
            vma: vma,
            machine_info: mac_info,
        }*/
    }

    #[inline]
    pub fn lookup_pg_table(&self, virt: VirtAddrType) -> Option<PhyAddrType> {
        #[cfg(feature = "prefetch")]
        {
            self.page_table
                .translate(VirtAddr::new(virt))
                .map(|v| v.as_u64())
        }

        #[cfg(not(feature = "prefetch"))]
        self.page_table.translate(virt)
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
    #[cfg(not(feature = "prefetch"))]
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

    #[cfg(feature = "prefetch")]
    /// Resume one page at remote side
    /// It will also prefetch adjacent pages if necessary
    ///
    /// This function is a little complex.
    /// It will handle prefetch stuffs during the polling process
    ///
    /// @param remote_va: remote virt-addr
    /// @param access_info: remote network meta info
    #[inline]
    pub unsafe fn read_remote_page(
        &self,
        remote_va: VirtAddrType,
        access_info: &AccessInfo,
    ) -> Option<*mut crate::bindings::page> {
        crate::log::info!("In prefetcher!");
        let (pt, idx) = self.page_table.find_l1_page_idx(VirtAddr::new(remote_va))?;
        let l1_page = &mut (*pt);

        let remote_pa = l1_page[idx];

        let new_page_p = crate::bindings::pmem_alloc_page(crate::bindings::PMEM_GFP_HIGHUSER);
        let new_page_pa = crate::bindings::pmem_page_to_phy(new_page_p) as u64;

        /*
        let res = crate::remote_paging::RemotePagingService::remote_read(
            new_page_pa,
            remote_pa,
            4096,
            access_info,
        ); */

        // FIXME: currently this code is from the remote_mapping.rs
        // But we need to trigger much code in it
        let res = {
            use crate::remote_paging::{DCReqPayload, TIMEOUT_USEC};
            use crate::KRdmaKit::rust_kernel_rdma_base::bindings::*;
            use core::pin::Pin;
            use os_network::timeout::TimeoutWRef;
            use os_network::{block_on, Conn};

            let pool_idx = crate::bindings::pmem_get_current_cpu() as usize;
            let (dc_qp, lkey) = crate::get_dc_pool_service_mut()
                .get_dc_qp_key(pool_idx)
                .expect("failed to get DCQP");

            let mut payload = DCReqPayload::default()
                .set_laddr(new_page_pa)
                .set_raddr(remote_pa) // copy from src into dst
                .set_sz(4096)
                .set_lkey(*lkey)
                .set_rkey(access_info.rkey)
                .set_send_flags(ib_send_flags::IB_SEND_SIGNALED)
                .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
                .set_ah_ptr(access_info.ah.get_inner())
                .set_dc_access_key(access_info.dct_key as _)
                .set_dc_num(access_info.dct_num);

            let mut payload = Pin::new_unchecked(&mut payload);
            os_network::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());

            // now sending the RDMA request
            dc_qp.post(&payload.as_ref()).unwrap(); // FIXME: it should never fail

            // Note, we do the prefetch things here
            // This can overlap with the networking requests latency
            // find prefetch pages
            let page_iter = RemotePageTableIter::new_from_l1(pt, idx);
            for (i, page) in page_iter.enumerate() {
                if i >= 2 {
                    break;
                }
                crate::log::info!("Test prefetch physical address {:?}", page);
            }

            // wait for the request to complete
            let mut timeout_dc = TimeoutWRef::new(dc_qp, TIMEOUT_USEC);
            match block_on(&mut timeout_dc) {
                Ok(_) => Ok(()),
                Err(e) => {
                    if e.is_elapsed() {
                        // The fallback path? DC cannot distinguish from failures
                        // unimplemented!();
                        crate::log::error!("fatal, timeout on reading the DC QP");
                        Err(os_network::rdma::Err::Timeout)
                        //Ok(())
                    } else {
                        Err(e.into_inner().unwrap())
                    }
                }
            }
        };

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
    ///
    /// **TODO**
    /// - Currently, we don't check the buf len, so this function is **unsafe**
    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        // FIXME: check buf len

        let mut cur = unsafe { bytes.truncate_header(0).unwrap() };

        // regs
        let regs = RegDescriptor::deserialize(&cur)?;
        cur = unsafe { cur.truncate_header(regs.serialization_buf_len())? };

        // VMA page counts
        let mut count: usize = 0;
        let off = unsafe { cur.memcpy_deserialize(&mut count)? };
        cur = unsafe { cur.truncate_header(off)? };

        crate::log::debug!("!!!!! start to deserialize vma, count: {}", count);

        // VMA & its corresponding page table
        #[cfg(not(feature = "prefetch"))]
        let mut pt = FlatPageTable::new();

        #[cfg(feature = "prefetch")]
        let mut pt = RemotePageTable::new();

        let mut vmas = Vec::new();

        for _ in 0..count {
            let vma = VMADescriptor::deserialize(&cur)?;
            cur = unsafe { cur.truncate_header(vma.serialization_buf_len())? };

            let vma_start = vma.get_start();
            vmas.push(vma);

            /*
            let vma_pg_table = CompactPageTable::deserialize(&cur)?;
            cur = unsafe { cur.truncate_header(vma_pg_table.serialization_buf_len())? };
            crate::log::info!("vma_pg_table len: {}", vma_pg_table.table_len()); */

            // now, deserialize the page table of this VMA
            // we don't use the `deserialize` method in the compact page table,
            // because it will incur unnecessary memory copies that is not optimal for the performance
            let mut page_num: usize = 0;
            let off = unsafe { cur.memcpy_deserialize(&mut page_num)? };

            cur = unsafe { cur.truncate_header(off)? };

            // crate::log::debug!("check page_num: {}", page_num);
            /*
            if page_num > 1024 {
                return None;
            }*/

            if core::mem::size_of::<Offset>() < core::mem::size_of::<VirtAddrType>()
                && page_num % 2 == 1
            {
                let mut pad: u32 = 0;
                let off = unsafe { cur.memcpy_deserialize(&mut pad)? };
                cur = unsafe { cur.truncate_header(off)? };
            }

            for _ in 0..page_num {
                let virt: Offset = unsafe { cur.read_unaligned_at_head() };
                cur = unsafe { cur.truncate_header(core::mem::size_of::<Offset>())? };

                let phy: Value = unsafe { cur.read_unaligned_at_head() };
                cur = unsafe { cur.truncate_header(core::mem::size_of::<Value>())? };

                #[cfg(not(feature = "prefetch"))]
                pt.add_one(virt as VirtAddrType + vma_start, phy);

                #[cfg(feature = "prefetch")]
                pt.map(
                    VirtAddr::new(virt as VirtAddrType + vma_start),
                    PhysAddr::new(phy),
                );
            }
        }

        let machine_info = RDMADescriptor::deserialize(&cur)?;

        Some(Self {
            regs: regs,
            page_table: pt,
            vma: vmas,
            machine_info: machine_info,
        })
    }

    fn serialization_buf_len(&self) -> usize {
        unimplemented!();
        /*
        self.regs.serialization_buf_len()
            + self.page_table.serialization_buf_len()
            + core::mem::size_of::<usize>() // the number of VMA descriptors
            + self.vma.len() * core::mem::size_of::<VMADescriptor>()
            + self.machine_info.serialization_buf_len() */
    }
}
