use kernel_module_testlib::{dmesg_contains, with_kernel_module};

#[test]
fn test_remote_page_table() {
    // a dummy test func
    with_kernel_module(|| {
        assert_eq!(dmesg_contains(&String::from("ERROR")), false);
    });
}
