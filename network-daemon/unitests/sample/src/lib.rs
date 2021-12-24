#![no_std]

struct SampleTestModule {
}

use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;
use linux_kernel_module::println;

impl linux_kernel_module::KernelModule for SampleTestModule {
    fn init() -> linux_kernel_module::KernelResult<Self> {
        println!("sample test module in raw kernel rdma bindings!");

        println!("check version");
        Ok(Self {})
    }
}

linux_kernel_module::kernel_module!(
    SampleTestModule,
    author: b"xmm",
    description: b"A sample module for unit testing",
    license: b"GPL"
);
