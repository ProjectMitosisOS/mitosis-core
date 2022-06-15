use std::env;
use std::path::PathBuf;

// types from customized syscalls
const INCLUDED_TYPES: &[&str] = &[];

// types from kernel
const INCLUDED_KERNEL_TYPES: &[&str] = &[];

const INCLUDED_KERNEL_FUNCS: &[&str] = &[];

const INCLUDED_VARS: &[&str] = &[];

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

    // do not compile in test mode (cargo test)
    let cargo_test_dir = env::var("CARGO_TARGET_DIR").expect("CARGO_TARGET_DIR missing, strange?");
    if cargo_test_dir.contains("test") {
        return;
    }

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

    println!("cargo:rerun-if-changed=src/kernel_helper.h");

    builder = builder.header("src/kernel_helper.h");

    // non-rust translatable type
    builder = builder.opaque_type("xregs_state");

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
        .write_to_file(out_path.join("bindings-rfork.rs"))
        .expect("Couldn't write bindings!");

    // build kernel_helper.c
    let mut builder = cc::Build::new();
    builder.compiler(env::var("CC").unwrap_or_else(|_| "clang".to_string()));
    builder.target(&target);
    builder.warnings(false);
    println!("cargo:rerun-if-changed=src/kernel_helper.c");

    builder.file("src/kernel_helper.c");

    for arg in kernel_args.iter() {
        builder.flag(&arg);
    }
    builder.compile("kernel_helper");
}
