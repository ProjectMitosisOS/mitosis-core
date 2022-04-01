use core::marker::PhantomData;

use crate::linux_kernel_module::bindings::*;
use crate::linux_kernel_module::c_types::*;
use crate::linux_kernel_module;

#[allow(unused_imports)]
use crate::linux_kernel_module::println;

pub use linux_kernel_module::file_operations::FileOperations;

mod basic;
pub use basic::*;

pub struct SysCallsHandler<T: FileOperations> {
    inner: T,
}

#[allow(dead_code)]
pub struct SysCallsService<T: FileOperations> {
    reg: linux_kernel_module::chrdev::Registration,
    _phantom: PhantomData<T>,
}

impl<T> SysCallsService<T>
where
    T: FileOperations,
{
    pub fn new() -> linux_kernel_module::KernelResult<Self> {
        use linux_kernel_module::cstr;
        let reg = linux_kernel_module::chrdev::builder(cstr!("mitosis"), 0..1)?
            .register_device::<SysCallsHandler<T>>(cstr!("mitosis-syscalls"))
            .build()?;
        Ok(Self {
            reg: reg,
            _phantom: PhantomData,
        })
    }
}

#[allow(non_upper_case_globals)]
impl<T> FileOperations for SysCallsHandler<T>
where
    T: FileOperations,
{
    #[inline]
    fn open(
        file: *mut crate::linux_kernel_module::bindings::file,
    ) -> crate::linux_kernel_module::KernelResult<Self> {
        let inner = T::open(file)?;
        Ok(Self { inner: inner })
    }

    #[inline]
    fn ioctrl(&mut self, cmd: c_uint, arg: c_ulong) -> c_long {
        self.inner.ioctrl(cmd, arg)
    }

    #[inline]
    fn mmap(&mut self, vma_p: *mut vm_area_struct) -> c_int {
        self.inner.mmap(vma_p)
    }
}
