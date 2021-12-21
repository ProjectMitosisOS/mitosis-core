use kernel_module_testlib::{with_kernel_module,assert_dmesg_ends,assert_dmesg_contains, dmesg_contains};

#[test]
fn test_simple() {
    // a dummy test func
    with_kernel_module(|| {
        println!("sampe test");

        let test_strs = ["bindings!".as_bytes()];
        assert_dmesg_ends(&test_strs);

        let test_strs2 = [String::from("raw kernel")];
        assert_dmesg_contains(&test_strs2);

        assert_eq!(dmesg_contains(&String::from("unknown")),false);
    });
}
