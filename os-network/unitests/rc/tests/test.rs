use kernel_module_testlib::{with_kernel_module,assert_dmesg_ends,assert_dmesg_contains, dmesg_contains};

#[test]
fn test_basic() {
    // a dummy test func
    with_kernel_module(|| {
        println!("basic");
        assert_eq!(dmesg_contains(&String::from("error")),false);
    });
}
