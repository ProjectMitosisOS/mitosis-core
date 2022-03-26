use kernel_module_testlib::{dmesg_contains, with_kernel_module};
use mitosis_rust_client::*;

#[test]
fn test_different_calls() {
    // a dummy test func
    with_kernel_module(|| {
        let mut client = MClientOptions::new()
            .set_device_name(DEFAULT_SYSCALL_PATH.to_string())
            .open()
            .unwrap();

        // test the task
        client.test(0).unwrap();
        assert_eq!(dmesg_contains(&String::from("test task")), true);

        // test the mm
        client.test(1).unwrap();
        assert_eq!(dmesg_contains(&String::from("test mm")), true);     
        
        // test the vma
        client.test(3).unwrap();
        assert_eq!(dmesg_contains(&String::from("test vma")), true);

        assert_eq!(dmesg_contains(&String::from("ERROR")), false);
    });
}
