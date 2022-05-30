use alloc::boxed::Box;

use crate::linux_kernel_module::mutex::LinuxMutex;

/// A simple wrapper over LinuxMutex to simplifiy creation
pub struct LockBundler<T> {
    inner: LinuxMutex<T>,
}

impl<T> LockBundler<T> {
    #[inline]
    pub fn new(inner: T) -> Box<Self> {
        let mut res = Box::new(Self {
            inner: LinuxMutex::new(inner),
        });

        res.inner.init();
        res
    }
}

#[allow(dead_code)]
impl<'a, T: 'a> LockBundler<T> {
    #[inline]
    pub fn lock<R>(&self, f: impl FnOnce(&'a T) -> R) -> R {
        self.inner.lock_f(f)
    }
}
