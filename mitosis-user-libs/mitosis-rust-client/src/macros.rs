/// *Credits*: This code is inspired & simplified from https://docs.rs/nix/0.9.0/nix/sys/ioctl/index.html
/// Generates a wrapper function for ioctl
/// The arguments to this macro are:
///
/// * The function name
/// * The ioctl request code
/// * The ioctl argument in pointer
///
/// The generated function has the following signature:
///
/// ```rust,ignore
/// pub struct TestArg { data : u64} 
/// pub unsafe fn FUNCTION_NAME(fd: libc::c_int,arg : *const TestArg) -> Result<libc::c_int>
/// ```
///
///
#[macro_export]
macro_rules! ioctl_write {
    ($(#[$attr:meta])* $name:ident, $nr:expr, $ty:ty) => (
        $(#[$attr])*
        pub unsafe fn $name(fd: $crate::libc::c_int,
                            data: *const $ty)
                            -> $crate::nix::Result<$crate::libc::c_int> {
            $crate::nix::errno::Errno::result($crate::libc::ioctl(fd,  $nr, data))
        }
    )
}

/// write version of the above code
#[macro_export]
macro_rules! ioctl_read {
    ($(#[$attr:meta])* $name:ident, $nr:expr, $ty:ty) => (
        $(#[$attr])*
        pub unsafe fn $name(fd: $crate::libc::c_int,
                            data: *mut $ty)
                            -> $crate::nix::Result<$crate::libc::c_int> {
            $crate::nix::errno::Errno::result($crate::libc::ioctl(fd,  $nr, data))
        }
    )
}

#[macro_export]
macro_rules! ioctl_test {
    ($(#[$attr:meta])* $name:ident, $ty:ty) => (
        $(#[$attr])*
        pub unsafe fn $name(fd: $crate::libc::c_int,
                            nr : u64,
                            data: *const $ty)
                            -> $crate::nix::Result<$crate::libc::c_int> {
            $crate::nix::errno::Errno::result($crate::libc::ioctl(fd,  nr, data))
        }
    )
}

#[cfg(test)]
mod tests {

    #[allow(unused_imports)]    
    use super::*;

    #[allow(dead_code)]
    pub struct TestArg { data : u64} 

    #[allow(dead_code)]
    ioctl_write!(test_arg, 0, TestArg); 

    #[test]
    fn ioctrl_macros() {        
    }
}
