#include "./kernel_helper.h"

#include <asm/pgalloc.h>
#include <linux/sched.h>
#include <linux/sched/task_stack.h>

#include <linux/ptrace.h>
#include <linux/cpumask.h>
#include <linux/smp.h>

struct thread_info*
pmem_get_current_thread_info(void)
{
  return current_thread_info();
}

struct task_struct*
pmem_get_current_task(void)
{
  return current;
}

int
pmem_call_walk_vma(struct vm_area_struct* vm, struct mm_walk* walk)
{
  static int (*walk_vma_range)(struct vm_area_struct * vm,
                               struct mm_walk * walk) = NULL;
  if (!walk_vma_range)
    walk_vma_range = (void*)kallsyms_lookup_name("walk_page_vma");
  return (*walk_vma_range)(vm, walk);
}

int
pmem_call_walk_range(unsigned long addr,
                     unsigned long end,
                     struct mm_walk* walk)
{
  static int (*walk_page_range)(
    unsigned long addr, unsigned long end, struct mm_walk* walk) = NULL;
  if (!walk_page_range)
    walk_page_range = (void*)kallsyms_lookup_name("walk_page_range");
  return (*walk_page_range)(addr, end, walk);
}

void
pmem_flush_tlb_all(void)
{
  static void (*k_flush_tlb_all)(void) = NULL;
  if (!k_flush_tlb_all)
    k_flush_tlb_all = (void*)kallsyms_lookup_name("flush_tlb_all");
  (*k_flush_tlb_all)();
}

long
pmem_do_arch_prctl_64(struct task_struct *task, int option, unsigned long arg2)
{
  static long (*do_arch_prctl_64)(struct task_struct *task, int option, unsigned long arg2) = NULL;
  if (!do_arch_prctl_64)
    do_arch_prctl_64 = (void*)kallsyms_lookup_name("do_arch_prctl_64");
  return (*do_arch_prctl_64)(task, option, arg2);
}

pte_t*
pmem_get_pte(struct mm_struct* mm, unsigned long addr)
{
  pgd_t* pgd;
  p4d_t* p4d;
  pud_t* pud;
  pmd_t* pmd;
  pte_t* pte;

  pgd = pgd_offset(mm, addr);
  if (pgd_none(*pgd) || pgd_bad(*pgd))
    return 0;

  p4d = p4d_offset(pgd, addr);
  if (p4d_none(*p4d) || p4d_bad(*p4d))
    return 0;

  pud = pud_offset(p4d, addr);
  if (pud_none(*pud) || pud_bad(*pud))
    return 0;

  pmd = pmd_offset(pud, addr);
  if (pmd_none(*pmd) || pmd_bad(*pmd))
    return 0;
  pte = pte_offset_map(pmd, addr);
  if (unlikely(pte_none(*pte)))
    return 0;
  return pte;
}

void
pmem_clear_pte_present(pte_t* pte)
{
  pte_t temp_pte;

  if (!pte_present(*pte)) {
    return;
  }
  temp_pte = pte_clear_flags(*pte, _PAGE_PRESENT);
  set_pte(pte, temp_pte);
}

unsigned long
pmem_get_phy_from_pte(pte_t* pte)
{
  return pte_val(*pte) & PAGE_MASK;
}

unsigned long
pmem_vm_mmap(struct file* file,
             unsigned long addr,
             unsigned long len,
             unsigned long prot,
             unsigned long flag,
             unsigned long offset)
{
  return vm_mmap(file, addr, len, prot, flag, offset);
}

unsigned long
pmem_mmap_region(struct file* file,
                 unsigned long addr,
                 unsigned long len,
                 vm_flags_t vm_flags,
                 unsigned long pgoff)
{
  return 0; // FIXME: no need such function right now
}

int
pmem_do_munmap(struct mm_struct* mm,
               unsigned long start,
               size_t len,
               struct list_head* uf)
{
  static int (*func)(struct mm_struct * mm,
                     unsigned long start,
                     size_t len,
                     struct list_head* uf) = NULL;
  if (!func)
    func = (void*)kallsyms_lookup_name("do_munmap");
  return (*func)(mm, start, len, uf);
}

struct pt_regs*
pmem_get_current_pt_regs(void)
{
  return current_pt_regs();
}

// https://stackoverflow.com/questions/6611346/how-are-the-fs-gs-registers-used-in-linux-amd64
// fs register is used to store the address of some user-space
// thread-local structures including the stack canary
unsigned long
pmem_arch_get_my_fs()
{
  unsigned long fsbase;
  rdmsrl(MSR_FS_BASE, fsbase);
  return fsbase;
}

// gs is used to store the address of some kernel-space
// thread-local structures
unsigned long
pmem_arch_get_my_gs()
{
  unsigned long gsbase;
  rdmsrl(MSR_KERNEL_GS_BASE, gsbase);
  return gsbase;
}

// set the fs register
long
pmem_arch_set_my_fs(unsigned long fsbase)
{
  return pmem_do_arch_prctl_64(current, ARCH_SET_FS, fsbase);
}

// set the gs register
long
pmem_arch_set_my_gs(unsigned long gsbase)
{
  return pmem_do_arch_prctl_64(current, ARCH_SET_GS, gsbase);
}

struct page*
pmem_alloc_page(gfp_t gfp_mask)
{
  return alloc_page(gfp_mask);
}

void
pmem_free_page(struct page* p)
{
  return __free_page(p);
}

u64
pmem_page_to_phy(struct page* page)
{
  return page_to_phys(page);
}

u64
pmem_page_to_virt(struct page* page)
{
    return page_to_virt(page);
}

void*
pmem_phys_to_virt(u64 p)
{
  return __va(p);
}

unsigned int
pmem_get_cpu_count(void)
{
  return nr_cpu_ids;
}

unsigned int
pmem_get_current_cpu(void)
{
  return smp_processor_id();
}

unsigned int
pmem_get_cpu(void)
{
  return get_cpu();
}

unsigned int
pmem_put_cpu(void)
{
  put_cpu();
  return 0;
}

unsigned int
pmem_filemap_fault(struct vm_fault *vmf){
    return filemap_fault(vmf);
}


// file related 
#include <linux/file.h>

void pmem_get_file(struct file *f) {
  get_file(f);
}

void pmem_put_file(struct file *f) {
  fput(f);
}

static inline void page_free_rmap(struct page *page, bool compound)
{
    atomic_dec(compound ? compound_mapcount_ptr(page) : &page->_mapcount);
}

static inline void page_dup_rmap(struct page *page, bool compound)
{
    atomic_inc(compound ? compound_mapcount_ptr(page) : &page->_mapcount);
}

void pmem_page_dup_rmap(struct page *page, bool compound) {
  page_dup_rmap(page, compound);
}

void pmem_page_free_rmap(struct page *page, bool compound) {
  page_free_rmap(page, compound);
}

void pmem_get_page(struct page *page) {
  return get_page(page);
}

void pmem_put_page(struct page *page) {
  return put_page(page);
}

