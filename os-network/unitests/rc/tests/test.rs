use kernel_module_testlib::{with_kernel_module, dmesg_contains};

#[test]
fn test_basic() {
    // a dummy test func
    with_kernel_module(|| {
        println!("basic");
        assert_eq!(dmesg_contains(&String::from("error")),false);
    });
}
