use alloc::boxed::Box;

use crate::linux_kernel_module::mutex::LinuxMutex;
use rust_kernel_linux_util::linux_kernel_module::sync::Mutex;

/// A simple wrapper over LinuxMutex to simplifiy creation
pub struct LockBundler<T> {
    inner: LinuxMutex<T>,
}

impl<T> LockBundler<T> {
    #[inline]
    pub fn new(inner: T) -> Box<Self> {
        let res = Box::new(Self {
            inner: LinuxMutex::new(inner),
        });

        res.inner.init();
        res
    }
}

#[allow(dead_code)]
impl<'a, T: 'a> LockBundler<T> {
    #[inline]
    pub fn lock<R>(&self, f: impl FnOnce(&'a mut T) -> R) -> R {
        self.inner.lock_f(f)
    }
}
