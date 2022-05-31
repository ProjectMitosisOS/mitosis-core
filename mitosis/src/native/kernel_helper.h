#ifndef OS_SWAP_KERNEL_HELPER_H
#define OS_SWAP_KERNEL_HELPER_H

#include <linux/kallsyms.h>
#include <linux/mm.h>
#include <linux/mm_types.h>
#include <linux/mman.h>
#include <linux/sched.h>
#include <linux/thread_info.h>

#include <asm/pgtable_types.h>
#include <asm/tlb.h>
#include <linux/vmalloc.h>
#include <asm/tlbflush.h>

#include <linux/gfp.h>

// dirty hack, because arch/x86/include/uapi/asm/prctl.h is not accessible when compiling kernel module
#ifndef ARCH_SET_GS
#define ARCH_SET_GS 0x1001
#endif
#ifndef ARCH_SET_FS
#define ARCH_SET_FS 0x1002
#endif

// handles inline functions
struct thread_info *
pmem_get_current_thread_info(void);

struct task_struct *
pmem_get_current_task(void);

pte_t *
pmem_get_pte(struct mm_struct *mm, unsigned long addr);

int pmem_call_walk_range(unsigned long addr,
                         unsigned long end,
                         struct mm_walk *walk);

int pmem_call_walk_vma(struct vm_area_struct *vm, struct mm_walk *walk);

unsigned long
pmem_get_phy_from_pte(pte_t *pte);

unsigned long
pmem_vm_mmap(struct file *file,
             unsigned long addr,
             unsigned long len,
             unsigned long prot,
             unsigned long flag,
             unsigned long offset);

int pmem_do_munmap(struct mm_struct *mm,
                   unsigned long start,
                   size_t len,
                   struct list_head *uf);

unsigned long
pmem_mmap_region(struct file *file,
                 unsigned long addr,
                 unsigned long len,
                 vm_flags_t vm_flags,
                 unsigned long pgoff);

void pmem_flush_tlb_range(struct vm_area_struct *vma, unsigned long start, unsigned long end);

void pmem_flush_tlb_all(void);

void pmem_flush_tlb_mm(struct mm_struct *mm);

void pmem_clear_pte_present(pte_t *pte);

struct pt_regs *
pmem_get_current_pt_regs(void);

struct page *
pmem_alloc_page(gfp_t gfp_mask);

void pmem_free_page(struct page *p);

int pmem_vm_insert_page(struct vm_area_struct *vma, unsigned long addr,
                        struct page *page);
u64 pmem_page_to_phy(struct page *page);

u64 pmem_page_to_virt(struct page *page);

void *
pmem_phys_to_virt(u64 p);

unsigned int
pmem_filemap_fault(struct vm_fault *vmf);
/*
 Page protection flags
 */
const pteval_t PMEM_PAGE_PRESENT = _PAGE_PRESENT;
const pteval_t PMEM_PAGE_RW = _PAGE_RW;
const pteval_t PMEM_PAGE_USER = _PAGE_USER;

/*
 VM protection flags
 Taken from <include/linux/mm.h>
 */
const unsigned long PMEM_VM_STACK = VM_STACK;
const unsigned long PMEM_VM_READ = VM_READ;
const unsigned long PMEM_VM_WRITE = VM_WRITE;
const unsigned long PMEM_VM_EXEC = VM_EXEC;
const unsigned long PMEM_VM_SHARED = VM_SHARED;
const unsigned long PMEM_VM_DONTEXPAND = VM_DONTEXPAND;
const unsigned long PMEM_VM_MAYREAD = VM_MAYREAD;
const unsigned long PMEM_VM_MAYWRITE = VM_MAYWRITE;
const unsigned long PMEM_VM_MIXEDMAP = VM_MIXEDMAP;
const unsigned long PMEM_VM_GROWSDOWN = VM_GROWSDOWN;
const unsigned long PMEM_VM_GROWSUP = VM_GROWSUP;

// forbidden
const unsigned long PMEM_VM_RESERVE = 0x00000000; // This flags seems not used by the Linux, use it

/*
 MMap flags
*/
const unsigned long PMEM_PROT_READ = PROT_READ;
const unsigned long PMEM_PROT_WRITE = PROT_WRITE;
const unsigned long PMEM_PROT_EXEC = PROT_EXEC;
const unsigned long PMEM_PROT_GROWSUP = PROT_GROWSUP;
const unsigned long PMEM_PROT_GROWSDOWN = PROT_GROWSDOWN;

/*
 Page fault flags
 */
const unsigned int PMEM_VM_FAULT_SIGSEGV = VM_FAULT_SIGSEGV;

/*
 gfp related
 */
const gfp_t PMEM_GFP_HIGHUSER = GFP_HIGHUSER;
const gfp_t PMEM_GFP_USER = GFP_USER;

/*
 About fs and gs
*/

unsigned long pmem_arch_get_my_fs(void);
unsigned long pmem_arch_get_my_gs(void);
long pmem_arch_set_my_fs(unsigned long fsbase);
long pmem_arch_set_my_gs(unsigned long gsbase);

/*
 CPU related
*/

unsigned int pmem_get_cpu_count(void);
unsigned int pmem_get_current_cpu(void);
unsigned int pmem_get_cpu(void);
unsigned int pmem_put_cpu(void);

/*
 file related
 */

void pmem_get_file(struct file *f);

void pmem_put_file(struct file *f);

void print_file_path(struct file *file);

/*
  page related
*/
void pmem_get_page(struct page *page);
void pmem_put_page(struct page *page);

void pmem_page_dup_rmap(struct page *page, bool compound);

void pmem_page_free_rmap(struct page *page, bool compound);

void pmem_clear_pte_write(pte_t *pte);
unsigned int pmem_check_pte_write(pte_t *pte);
struct page *
pmem_pte_to_page(pte_t *pte);

#endif
