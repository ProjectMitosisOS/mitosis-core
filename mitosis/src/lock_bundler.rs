use crate::linux_kernel_module::mutex::LinuxMutex;
use alloc::boxed::Box;
use rust_kernel_linux_util::linux_kernel_module::sync::Mutex;

pub type BoxedLockBundler<T> = Box<LockBundler<T>>;

/// A simple wrapper over LinuxMutex to simplifiy creation
pub struct LockBundler<T> {
    inner: LinuxMutex<T>,
}

impl<T> LockBundler<T> {
    // `LinuxMutex` is wrapped in a `Box` because it is a self-referening struct that 
    // cannot be moved.
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
