use kernel_module_testlib::{with_kernel_module, dmesg_contains};

#[test]
fn test_global() {
    // a dummy test func
    with_kernel_module(|| {
    });
}
