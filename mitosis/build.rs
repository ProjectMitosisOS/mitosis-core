use std::env;
use std::path::PathBuf;

// types from customized syscalls
const INCLUDED_ENUMS: &[&str] = &[
    "LibMITOSISCmd"
];

const INCLUDED_TYPES: &[&str] = &[
    "connect_req_t",
    "resume_remote_req_t"
];

// types from kernel
const INCLUDED_KERNEL_TYPES: &[&str] = &[
    "task_struct",
    "thread_info",
    "mm_struct",
    "vm_area_struct",
    "vm_operations_struct",
    "mm_walk",
    "vm_flags_t",
    "pgprot_t",
    "file_system_type",
    "timespec",
];
const INCLUDED_KERNEL_FUNCS: &[&str] = &[
    "print_file_path",
    "pmem_mmap_region",
    "pmem_vm_mmap",
    "pmem_get_phy_from_pte",
    "pmem_call_walk_range",
    "pmem_call_walk_vma",
    "pmem_get_current_thread_info",
    "pmem_get_current_task",
    "pmem_get_pte",
    "pmem_flush_tlb_mm",
    "pmem_flush_tlb_all",
    "pmem_flush_tlb_range",
    "pmem_clear_pte_present",
    "pmem_clear_pte_write",
    "pmem_check_pte_present",
    "pmem_check_pte_write",
    "pmem_set_pte_write",
    "pmem_pte_to_page",
    "find_vma",
    "pmem_alloc_page",
    "pmem_free_page",
    "get_zeroed_page",
    "pmem_page_to_phy",
    "pmem_page_to_virt",
    "pmem_phys_to_virt",
    "pmem_vm_insert_page",
    "memcpy",
    // mmap related
    "pmem_do_munmap",
    "vm_munmap",
    "pmem_get_current_pt_regs",
    // fs, gs related
    "pmem_arch_get_my_fs",
    "pmem_arch_get_my_gs",
    "pmem_arch_set_my_fs",
    "pmem_arch_set_my_gs",
    // file_operations related
    "no_llseek",
    // cpu related
    "pmem_get_cpu_count",
    "pmem_get_current_cpu",
    "pmem_get_cpu",
    "pmem_put_cpu",
    "pmem_filemap_fault",
    "pmem_get_file",
    "pmem_put_file",
    "schedule",
    // vmalloc, vfree
    "vmalloc",
    "vfree",

    // page related
    "pmem_get_page",
    "pmem_put_page",
    "pmem_page_dup_rmap",
    "pmem_page_free_rmap",

    // malloc
    "vmalloc",
    "vfree",

    "pmem_getnstimeofday",
];

const INCLUDED_VARS: &[&str] = &[
    "O_NONBLOCK",
    "PMEM_PAGE_PRESENT",
    "PMEM_PAGE_RW",
    "PMEM_PAGE_USER",
    "PMEM_PAGE_NX",
    "PMEM_VM_STACK",
    "PMEM_VM_READ",
    "PMEM_VM_WRITE",
    "PMEM_VM_MAYREAD",
    "PMEM_VM_MAYWRITE",
    "PMEM_VM_GROWSDOWN",
    "PMEM_VM_GROWSUP",
    "PMEM_VM_RESERVE",
    "PMEM_VM_EXEC",
    "PMEM_VM_SHARED",
    "PMEM_VM_STACK",
    "PMEM_VM_DONTEXPAND",
    "PMEM_VM_MIXEDMAP",
    "PMEM_PROT_READ",
    "PMEM_PROT_WRITE",
    "PMEM_PROT_EXEC",
    "PMEM_PROT_GROWSUP",
    "PMEM_VM_FAULT_SIGSEGV",
    "PMEM_GFP_HIGHUSER",
    "PMEM_GFP_USER"
];

// Takes the CFLAGS from the kernel Makefile and changes all the include paths to be absolute
// instead of relative.
fn prepare_cflags(cflags: &str, kernel_dir: &str) -> Vec<String> {
    let cflag_parts = shlex::split(&cflags).unwrap();
    let mut cflag_iter = cflag_parts.iter();
    let mut kernel_args = vec![];
    while let Some(arg) = cflag_iter.next() {
        if arg.starts_with("-I") && !arg.starts_with("-I/") {
            kernel_args.push(format!("-I{}/{}", kernel_dir, &arg[2..]));
        } else if arg == "-include" {
            kernel_args.push(arg.to_string());
            let include_path = cflag_iter.next().unwrap();
            if include_path.starts_with('/') {
                kernel_args.push(include_path.to_string());
            } else {
                kernel_args.push(format!("{}/{}", kernel_dir, include_path));
            }
        } else {
            kernel_args.push(arg.to_string());
        }
    }
    //    println!("!!! {:?}", kernel_args);
    kernel_args
}

fn main() {
    println!("cargo:rust-cfg=out");

    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=KDIR");
    println!("cargo:rerun-if-env-changed=c_flags");
    println!("cargo:rerun-if-env-changed=ofa_flags");

    let kernel_dir = env::var("KDIR").expect("Must be invoked from kernel makefile");
    let kernel_cflags = env::var("c_flags").expect("Add 'export c_flags' to Kbuild");
    let kbuild_cflags_module =
        env::var("KBUILD_CFLAGS_MODULE").expect("Must be invoked from kernel makefile");

    let ofa_flags = env::var("ofa_flags").expect("Add extra ofa flags to Kbuild");

    let cflags = format!("{} {} {}", ofa_flags, kernel_cflags, kbuild_cflags_module);
    let kernel_args = prepare_cflags(&cflags, &kernel_dir);

    let target = env::var("TARGET").unwrap();

    let mut builder = bindgen::Builder::default()
        .use_core()
        .ctypes_prefix("c_types")
        .derive_default(true)
        .size_t_is_usize(true)
        .rustfmt_bindings(true);

    builder = builder.clang_arg(format!("--target={}", target));
    for arg in kernel_args.iter() {
        builder = builder.clang_arg(arg.clone());
    }

    println!("cargo:rerun-if-changed=src/native/kernel_helper.h");

    builder = builder
        .header("src/native/kernel_helper.h")
        .whitelist_function("pmem_*");

    println!("cargo:rerun-if-changed=../mitosis-user-libs/mitosis-c-client/include/common.h");
    builder = builder.header("../mitosis-user-libs/mitosis-c-client/include/common.h");
    // non-rust translatable type
    builder = builder.opaque_type("xregs_state");

    for t in INCLUDED_ENUMS {
        builder = builder.whitelist_type(t);
        builder = builder.constified_enum_module(t);
    }

    for t in INCLUDED_TYPES {
        builder = builder.whitelist_type(t);
    }

    for t in INCLUDED_KERNEL_TYPES {
        builder = builder.whitelist_type(t);
    }

    for f in INCLUDED_KERNEL_FUNCS {
        builder = builder.whitelist_function(f);
    }

    for v in INCLUDED_VARS {
        builder = builder.whitelist_var(v);
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-mitosis-core.rs"))
        .expect("Couldn't write bindings!");

    // build kernel_helper.c
    let mut builder = cc::Build::new();
    builder.compiler(env::var("CC").unwrap_or_else(|_| "clang".to_string()));
    builder.target(&target);
    builder.warnings(false);
    println!("cargo:rerun-if-changed=src/native/kernel_helper.c");

    builder.file("src/native/kernel_helper.c");

    for arg in kernel_args.iter() {
        builder.flag(&arg);
    }
    builder.compile("kernel_helper");
}
