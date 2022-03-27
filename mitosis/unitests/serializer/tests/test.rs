use kernel_module_testlib::{dmesg_contains, with_kernel_module};
use mitosis_rust_client::*;

#[test]
fn test_call_nil() {
    // a dummy test func
    with_kernel_module(|| {
        let mut client = MClientOptions::new()
            .set_device_name(DEFAULT_SYSCALL_PATH.to_string())
            .open()
            .unwrap();
        client.test(0).unwrap();
        client.test(1).unwrap();
        // client.test(3).unwrap();
        // client.test(4).unwrap();
        assert_eq!(dmesg_contains(&String::from("ERROR")), false);
    });
}
