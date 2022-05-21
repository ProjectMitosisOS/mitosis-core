#![no_std]

extern crate alloc;

use mitosis::linux_kernel_module;
use mitosis::log;

use mitosis::remote_mapping::*;

use krdma_test::*;

use alloc::boxed::Box;

fn test_basic() {
    let mut pt = Box::new(RemotePageTable::new());
    log::info!("in test basic page_table: {:?}, is empty {}", pt, pt.is_empty());
        
    let page = RemotePage::containing_address(VirtAddr::new(0xdeadbeaf));

    // map a single page
    assert!(pt.map(VirtAddr::new(4096), PhysAddr::new(73)).is_none());
    log::info!("in test basic page_table: {:?}, is empty {}", pt, pt.is_empty());

    log::info!("check lookup result {:?}", pt.translate(VirtAddr::new(0xdeadbeaf)));
    log::info!("check lookup result {:?}", pt.translate(VirtAddr::new(4096)));
}

#[krdma_test(test_basic)]
fn init() {
    log::info!("in test mitosis remote page table!");
}

#[krdma_drop]
fn clean() {
    //    end_instance();
}
