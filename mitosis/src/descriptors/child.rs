#[allow(unused_imports)]
use crate::bindings::page;
use crate::linux_kernel_module;
use alloc::vec::Vec;
#[allow(unused_imports)]
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};

use os_network::bytes::BytesMut;
#[allow(unused_imports)]
use os_network::future::{Async, Future};
use os_network::Conn;

use super::rdma::RDMADescriptor;
use super::reg::RegDescriptor;
use super::vma::VMADescriptor;

#[allow(unused_imports)]
use super::page_table::FlatPageTable;

#[allow(unused_imports)]
use crate::remote_mapping::{PageEntry, PhysAddr, RemotePageTable, RemotePageTableIter, VirtAddr};

#[allow(unused_imports)]
use super::parent::{CompactPageTable, Offset, Value};

use crate::kern_wrappers::mm::{PhyAddrType, VirtAddrType};
use crate::kern_wrappers::task::Task;
use crate::remote_paging::AccessInfo;

#[cfg(feature = "prefetch")]
use crate::prefetcher::{DCAsyncPrefetcher, StepPrefetcher};

/// The kernel-space process descriptor of MITOSIS
/// The descriptors should be generate by the task
#[allow(dead_code)]
pub struct ChildDescriptor {
    pub regs: RegDescriptor,

    // #[cfg(not(feature = "prefetch"))]
    // pub page_table: FlatPageTable,
    //
    // #[cfg(feature = "prefetch")]
    pub page_table: RemotePageTable,

    pub vma: Vec<VMADescriptor>,
    pub machine_info: RDMADescriptor,

    #[cfg(feature = "prefetch")]
    pub prefetcher: DCAsyncPrefetcher<'static>,
    #[cfg(feature = "eager-resume")]
    pub eager_fetched_pages: hashbrown::HashSet<VirtAddrType>,
    #[cfg(feature = "resume-profile")]
    pub remote_fetched_page_count: usize,
}

impl ChildDescriptor {
    /// # Warning: Deperacted
    /// The ChildDescriptor can only be built from the ParentDescriptor's
    /// serialzied buffer
    pub fn new(_task: &crate::kern_wrappers::task::Task, _mac_info: RDMADescriptor) -> Self {
        unimplemented!();
    }

    #[inline(always)]
    pub fn lookup_pg_table(&self, virt: VirtAddrType) -> Option<PhyAddrType> {
        self.page_table
            .translate(VirtAddr::new(virt))
            .map(|v| v.as_u64())
    }

    /// Apply the descriptor into current process
    #[inline]
    pub fn apply_to(&mut self, file: *mut crate::bindings::file) {
        let mut task = Task::new();
        // 1. Unmap origin vma regions
        task.unmap_self();
        let access_info = AccessInfo::new(&self.machine_info).unwrap();
        // let access_info = AccessInfo::new_from_cache(self.machine_info.mac_id, &self.machine_info).unwrap();

        // 2. Map new vma regions
        #[cfg(not(feature = "eager-resume"))]
        (&self.vma).into_iter().enumerate().for_each(|(i, m)| {
            let vma = unsafe { task.map_one_region(file, &m, self.vma.get(i + 1)) };

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
        });

        #[cfg(feature = "eager-resume")]
        self.vma.clone().into_iter().enumerate().for_each(|(i, m)| {
            let vma = unsafe { task.map_one_region(file, &m, self.vma.get(i + 1)) };

            #[allow(dead_code)]
            let vma = vma.unwrap();

            // tune the bits
            let origin_vma_flags =
                unsafe { crate::bindings::VMFlags::from_bits_unchecked(m.flags) };

            if origin_vma_flags.contains(crate::bindings::VMFlags::VM_ALLOC) {
                // set the vma
                crate::kern_wrappers::vma::VMA::new(vma).set_alloc();
            }
            #[cfg(feature = "eager-resume")]
            self.eager_fetch_vma(&m, vma, &access_info);
        });

        // 3. Re-set states
        task.set_mm_reg_states(&self.regs);
    }
}

impl ChildDescriptor {
    #[cfg(feature = "eager-resume")]
    fn eager_fetch_vma(
        &mut self,
        vma_des: &VMADescriptor,
        vma: &'static mut crate::bindings::vm_area_struct,
        access_info: &AccessInfo,
    ) {
        let (size, start) = (vma_des.get_sz(), vma_des.get_start());
        let len = 12;
        let mut addr_buf: Vec<VirtAddrType> = Vec::with_capacity(len);
        for addr in (start..start + size).step_by(4096) {
            if addr_buf.len() < len {
                addr_buf.push(addr);
            }
            if len == addr_buf.len() {
                // batch
                let page_list = self.batch_read_remote_pages(&addr_buf, access_info);

                for (i, new_page_p) in page_list.iter().enumerate() {
                    if let Some(new_page_p) = new_page_p {
                        vma.vm_page_prot.pgprot =
                            vma.vm_page_prot.pgprot | (((1 as u64) << 52) as u64); // present bit
                        let _ = unsafe {
                            crate::bindings::pmem_vm_insert_page(vma, addr_buf[i], *new_page_p)
                        };
                        self.eager_fetched_pages.insert(*new_page_p as VirtAddrType);
                    }
                }
                addr_buf.clear();
            }
        }
        if !addr_buf.is_empty() {
            // batch
            let page_list = self.batch_read_remote_pages(&addr_buf, access_info);

            for (i, new_page_p) in page_list.iter().enumerate() {
                if let Some(new_page_p) = new_page_p {
                    vma.vm_page_prot.pgprot = vma.vm_page_prot.pgprot | (((1 as u64) << 52) as u64); // present bit
                    let _ = unsafe {
                        crate::bindings::pmem_vm_insert_page(vma, addr_buf[i], *new_page_p)
                    };
                    self.eager_fetched_pages.insert(*new_page_p as VirtAddrType);
                }
            }
        }
    }

    #[inline]
    #[allow(dead_code)]
    fn batch_read_remote_pages(
        &self,
        addr_list: &Vec<VirtAddrType>,
        access_info: &AccessInfo,
    ) -> Vec<Option<*mut crate::bindings::page>> {
        let mut res: Vec<Option<*mut crate::bindings::page>> = Vec::with_capacity(addr_list.len());
        for (i, remote_va) in addr_list.iter().enumerate() {
            let remote_pa = self.lookup_pg_table(*remote_va);
            if remote_pa.is_none() {
                res.push(None);
                continue;
            }

            let new_page_p =
                unsafe { crate::bindings::pmem_alloc_page(crate::bindings::PMEM_GFP_HIGHUSER) };
            let new_page_pa = unsafe { crate::bindings::pmem_page_to_phy(new_page_p) } as u64;

            let result = {
                use crate::KRdmaKit::rust_kernel_rdma_base::bindings::*;
                let (dst, src, sz) = (new_page_pa, remote_pa.unwrap(), 4096);
                let flag = if i == addr_list.len() - 1 {
                    ib_send_flags::IB_SEND_SIGNALED
                } else {
                    0
                };
                let pool_idx = unsafe { crate::bindings::pmem_get_current_cpu() } as usize;
                let (dc_qp, lkey) =
                    unsafe { crate::get_dc_pool_service_mut().get_dc_qp_key(pool_idx) }
                        .expect("failed to get DCQP");

                let mut payload = crate::remote_paging::DCReqPayload::default()
                    .set_laddr(dst)
                    .set_raddr(PhysAddr::decode(src)) // copy from src into dst
                    .set_sz(sz as _)
                    .set_lkey(*lkey)
                    .set_rkey(access_info.rkey)
                    .set_send_flags(flag)
                    .set_opcode(ib_wr_opcode::IB_WR_RDMA_READ)
                    .set_ah_ptr(unsafe { access_info.ah.get_inner() })
                    .set_dc_access_key(access_info.dct_key as _)
                    .set_dc_num(access_info.dct_num);

                let mut payload = unsafe { core::pin::Pin::new_unchecked(&mut payload) };
                os_network::rdma::payload::Payload::<ib_dc_wr>::finalize(payload.as_mut());

                // now sending the RDMA request
                let res = dc_qp.post(&payload.as_ref());
                if res.is_err() {
                    crate::log::error!("failed to batch read pages {:?}", res);
                }

                if i == addr_list.len() - 1 {
                    // wait for the request to complete
                    let mut timeout_dc = os_network::timeout::TimeoutWRef::new(
                        dc_qp,
                        crate::remote_paging::TIMEOUT_USEC,
                    );
                    match os_network::block_on(&mut timeout_dc) {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            if e.is_elapsed() {
                                crate::log::error!("fatal, timeout on reading the DC QP");
                                Err(os_network::rdma::Err::Timeout)
                            } else {
                                Err(e.into_inner().unwrap())
                            }
                        }
                    }
                } else {
                    Ok(())
                }
            };

            match result {
                Ok(_) => res.push(Some(new_page_p)),
                Err(e) => {
                    crate::log::error!(
                        "[batch_read_remote_pages] Failed to read the remote page {:?}",
                        e
                    );
                    unsafe { crate::bindings::pmem_free_page(new_page_p) };
                    res.push(None)
                }
            }
        }

        return res;
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
        &mut self,
        remote_va: PhyAddrType,
        access_info: &AccessInfo,
    ) -> Option<*mut crate::bindings::page> {
        let remote_pa = self.lookup_pg_table(remote_va);
        if remote_pa.is_none() {
            return None;
        }
        let remote_pa = remote_pa.unwrap();
        let new_page_p = crate::bindings::pmem_alloc_page(crate::bindings::PMEM_GFP_HIGHUSER);
        let new_page_pa = crate::bindings::pmem_page_to_phy(new_page_p) as u64;
        let res = crate::remote_paging::RemotePagingService::remote_read(
            new_page_pa,
            remote_pa,
            4096,
            access_info,
        );
        #[cfg(feature = "resume-profile")]
        self.incr_fetched_remote_count(1);
        return match res {
            Ok(_) => Some(new_page_p),
            Err(e) => {
                crate::log::error!("Failed to read the remote page {:?}", e);
                crate::bindings::pmem_free_page(new_page_p);
                None
            }
        };
    }

    #[cfg(feature = "prefetch")]
    /// Resume one page at remote side
    ///
    /// @param remote_va: remote virt-addr
    /// @param access_info: remote network meta info
    #[inline]
    pub unsafe fn read_remote_page_wo_prefetch(
        &mut self,
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
        #[cfg(feature = "resume-profile")]
        self.incr_fetched_remote_count(1);
        return match res {
            Ok(_) => Some(new_page_p),
            Err(e) => {
                crate::log::error!("Failed to read the remote page {:?}", e);
                crate::bindings::pmem_free_page(new_page_p);
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
        &mut self,
        remote_va: VirtAddrType,
        access_info: &AccessInfo,
    ) -> Option<*mut crate::bindings::page> {
        let (pt, idx) = self.page_table.find_l1_page_idx(VirtAddr::new(remote_va))?;
        let l1_page = &mut (*pt);

        let mut remote_pa = l1_page[idx];

        // check whether it has been prefetched to local
        #[cfg(feature = "prefetch")]
        {
            // we need to do the prefetch
            if PhysAddr::new(remote_pa).is_prefetch() {
                /*
                Two cases.
                    1. the page is prefetched. then we can directly return
                    2. the page is prefetched, but the content has not ready.
                    In this case, we need to poll the connection to wait for it ready.

                FIXME:
                - Currently, we don't handle the timeout here
                 */
                while remote_pa == crate::remote_mapping::K_MAGIC_IN_PREFETCH {
                    // poll the prefetcher
                    self.poll_prefetcher();
                    remote_pa = l1_page[idx];
                    compiler_fence(SeqCst);
                }

                // The remote page is encoded in the page table as
                //     *mut addr | 1
                let page = PhysAddr::decode(remote_pa as _) as *mut page;

                // clean the entry in the page table, since
                // the OS is responsible for reclaiming this page
                l1_page[idx] = 0;

                return Some(page);
            }
        }

        let new_page_p = crate::bindings::pmem_alloc_page(crate::bindings::PMEM_GFP_HIGHUSER);
        let new_page_pa = crate::bindings::pmem_page_to_phy(new_page_p) as u64;

        // FIXME: currently this code is from the remote_mapping.rs
        // But we need to use this piece of code
        let res = {
            use crate::remote_paging::{DCReqPayload, TIMEOUT_USEC};
            use crate::KRdmaKit::rust_kernel_rdma_base::bindings::*;
            use core::pin::Pin;
            use os_network::block_on;
            use os_network::timeout::TimeoutWRef;

            let pool_idx = crate::bindings::pmem_get_current_cpu() as usize;
            let (dc_qp, lkey) = crate::get_dc_pool_service_mut()
                .get_dc_qp_key(pool_idx)
                .expect("failed to get DCQP");

            let mut payload = DCReqPayload::default()
                .set_laddr(new_page_pa)
                .set_raddr(PhysAddr::decode(remote_pa)) // copy from src into dst
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
            let pte_iter = RemotePageTableIter::new_from_l1(pt, idx);
            self.prefetcher.execute_reqs(
                pte_iter,
                StepPrefetcher::<PageEntry, { crate::PREFETCH_STEP }>::new(),
            );
            self.poll_prefetcher();

            // wait for the request to complete
            let mut timeout_dc = TimeoutWRef::new(dc_qp, TIMEOUT_USEC);
            #[cfg(feature = "resume-profile")]
            self.incr_fetched_remote_count(crate::PREFETCH_STEP + 1);
            match block_on(&mut timeout_dc) {
                Ok(_) => Ok(()),
                Err(e) => {
                    if e.is_elapsed() {
                        // The fallback path? DC cannot distinguish from failures
                        // unimplemented!();
                        crate::log::error!("fatal, timeout on reading the DC QP");
                        Err(os_network::rdma::Err::Timeout)
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
                crate::bindings::pmem_free_page(new_page_p);
                None
            }
        };
    }

    #[cfg(feature = "prefetch")]
    fn poll_prefetcher(&mut self) {
        loop {
            #[allow(non_snake_case)]
            match self.prefetcher.poll() {
                Ok(Async::Ready(_)) => {
                    // The second poll is likely to succeed
                    // so just continue
                }
                Ok(_NotReady) => {
                    return;
                }
                Err(_e) => panic!("not implemented"),
            }
        }
    }

    #[cfg(feature = "resume-profile")]
    fn incr_fetched_remote_count(&mut self, page_cnt: usize) {
        self.remote_fetched_page_count += page_cnt;
    }
}

impl os_network::serialize::Serialize for ChildDescriptor {
    fn serialize(&self, _bytes: &mut BytesMut) -> bool {
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
        let mut pt = RemotePageTable::new();

        let mut vmas = Vec::new();

        for _ in 0..count {
            let vma = VMADescriptor::deserialize(&cur)?;
            cur = unsafe { cur.truncate_header(vma.serialization_buf_len())? };

            let vma_start = vma.get_start();
            vmas.push(vma);

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

                pt.map(
                    VirtAddr::new(virt as VirtAddrType + vma_start),
                    PhysAddr::new(phy),
                );
            }
        }

        let machine_info = RDMADescriptor::deserialize(&cur)?;

        #[cfg(feature = "prefetch")]
        let (prefetch_conn, lkey) =
            unsafe { crate::get_dc_pool_async_service_ref().lock(|p| p.pop_one_qp())? };

        #[cfg(feature = "prefetch")]
        let access_info = AccessInfo::new(&machine_info);
        // let access_info = AccessInfo::new_from_cache(machine_info.mac_id, &machine_info);
        #[cfg(feature = "prefetch")]
        if access_info.is_none() {
            return None;
        }
        Some(Self {
            regs: regs,
            page_table: pt,
            vma: vmas,
            machine_info: machine_info,

            #[cfg(feature = "prefetch")]
            prefetcher: DCAsyncPrefetcher::new_from_raw(prefetch_conn, lkey, access_info.unwrap()),
            #[cfg(feature = "eager-resume")]
            eager_fetched_pages: Default::default(),
            #[cfg(feature = "resume-profile")]
            remote_fetched_page_count: 0
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
