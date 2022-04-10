use kernel_module_testlib::{dmesg_contains, with_kernel_module};
use mitosis_rust_client::*;

#[test]
fn test_large_prepare() {
    // a dummy test func
    with_kernel_module(|| {
        let mut client = MClientOptions::new()
            .set_device_name(DEFAULT_SYSCALL_PATH.to_string())
            .open()
            .unwrap();
        
        client.nil().unwrap();

        // prepare a very large image
        let mut vec = Vec::new();
        let MB = 1024 * 1024;
        let prepared_sz = MB * 1024;
        // let prepared_sz = 8 * MB;
        let entries = prepared_sz / std::mem::size_of::<u64>();
        
        for i in 0..entries { 
            vec.push(i);
        }

        client.prepare(73).unwrap();

        assert_eq!(dmesg_contains(&String::from("ERROR")), false);
    });
}
