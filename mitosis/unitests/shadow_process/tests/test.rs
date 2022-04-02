use kernel_module_testlib::{dmesg_contains, with_kernel_module};
use mitosis_rust_client::*;

#[test]
fn test_basic() {
    // a dummy test func
    with_kernel_module(|| {
        let mut client = MClientOptions::new()
            .set_device_name(DEFAULT_SYSCALL_PATH.to_string())
            .open()
            .unwrap();

        client.test(0).unwrap();    // basic        

        assert_eq!(dmesg_contains(&String::from("ERROR")), false);
    });
}


#[test]
fn test_page() {
    // a dummy test func
    with_kernel_module(|| {
        let mut client = MClientOptions::new()
            .set_device_name(DEFAULT_SYSCALL_PATH.to_string())
            .open()
            .unwrap();

        client.test(0).unwrap();    // basic

        let mut msg : [u8; 4096] = [0; 4096];
        msg[0] = 0x0;
        msg[1] = 0x1;
        msg[2] = 0x2;
        msg[3] = 0x3;
        msg[4] = 0x4;
        msg[5] = 0x5;
        msg[6] = 0x6;
        msg[7] = 0x7;
        msg[8] = 0x8;
        msg[9] = 0x0;
        msg[10] = 0xa;
        msg[11] = 0xb;
        msg[12] = 0xc;
        msg[13] = 0xd;
        msg[14] = 0xe;
        msg[15] = 0xf;

        client.test_w_arg(1, msg.as_ptr()).unwrap();

        assert_eq!(dmesg_contains(&String::from("ERROR")), false);
    });
}