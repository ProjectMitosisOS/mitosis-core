// A simple static bytes abstraction inspired by https://github.com/tokio-rs/bytes/
// In kernel, all written pointers are static
pub struct BytesMut {
    pub(crate) ptr: *mut u8,
    len: usize,
}

pub trait ToBytes {
    fn get_bytes(&self) -> &BytesMut;
    fn get_bytes_mut(&mut self) -> &mut BytesMut;
}

impl BytesMut {
    pub fn from_static(bytes: &'static mut [u8]) -> Self {
        Self {
            ptr: bytes.as_mut_ptr(),
            len: bytes.len(),
        }
    }

    pub unsafe fn from_raw(ptr: *mut u8, len: usize) -> Self {
        let slice = core::slice::from_raw_parts_mut(ptr, len);
        Self::from_static(slice)
    }

    pub unsafe fn truncate_header(&self, off: usize) -> core::option::Option<Self> {
        if self.len >= off {
            Some(Self {
                ptr: self.ptr.offset(off as isize),
                len: self.len - off,
            })
        } else {
            None
        }
    }    

    pub unsafe fn clone_and_resize(&self, sz : usize) -> core::option::Option<Self> { 
        if self.len >= sz { 
            Some(Self { 
                ptr : self.ptr, 
                len : sz
            })
        } else { 
            None
        }
    }

    pub unsafe fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
        }
    }
}

impl BytesMut {
    /// return true if the range of payload's len is within [ptr + offset, ptr + len]
    #[inline(always)]
    pub fn copy(&mut self, payload: &Self, offset: usize) -> bool {
        if core::intrinsics::likely(self.len.checked_sub(offset).is_some()) {
            unsafe { core::ptr::copy_nonoverlapping(payload.ptr, self.ptr, payload.len) };
            return true;
        }
        false
    }

    #[inline(always)]
    pub fn get_raw(&self) -> u64 {
        self.ptr as u64
    }

    pub fn resize(&mut self, sz: usize) {
        if core::intrinsics::likely(sz < self.len()) {
            self.len = sz;
        }
    }

    /// Write T to the current header, return the size of T
    /// This function is unsafe, because we don't check whether the 
    /// buffer has enough capacity to hold the data 
    pub unsafe fn write_unaligned_at_head<T>(&mut self, src : T) -> usize {     
        core::ptr::write_unaligned(self.ptr as *mut T, src);
        core::mem::size_of::<T>()
    }


    /// Read T at the current header
    /// This function is unsafe, because we don't check whether the 
    /// buffer has enough capacity to hold the data 
    pub unsafe fn read_unaligned_at_head<T>(&self) -> T {     
        core::ptr::read_unaligned(self.ptr as *const T)
    }

    pub unsafe fn get_ptr(&self) -> *mut u8 {
        self.ptr
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub unsafe fn at_unchecked(&self, offset: usize) -> u8 {
        core::ptr::read(self.ptr.offset(offset as isize))
    }
}

impl core::cmp::PartialEq for BytesMut {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        for i in 0..self.len {
            if unsafe { self.at_unchecked(i) != other.at_unchecked(i) } {
                return false;
            }
        }
        true
    }
}

impl core::cmp::Eq for BytesMut {}

use core::fmt::{Arguments, Debug, Formatter, Result, Write};
// use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module::println;

impl Write for BytesMut {
    #[inline]
    fn write_str(&mut self, s: &str) -> Result {
        if s.len() <= self.len {
            unsafe { core::ptr::copy_nonoverlapping(s.as_ptr(), self.ptr, s.len()) };
            Ok(())
        } else {
            Err(core::fmt::Error)
        }
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments<'_>) -> Result {
        core::fmt::write(self, args)
    }
}

impl Debug for BytesMut {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "b\"")?;
        for &b in unsafe { core::slice::from_raw_parts(self.ptr as *const u8, self.len) } {
            // https://doc.rust-lang.org/reference/tokens.html#byte-escapes
            if b == b'\n' {
                write!(f, "\\n")?;
            } else if b == b'\r' {
                write!(f, "\\r")?;
            } else if b == b'\t' {
                write!(f, "\\t")?;
            } else if b == b'\\' || b == b'"' {
                write!(f, "\\{}", b as char)?;
            } else if b == b'\0' {
                write!(f, "\\0")?;
            // ASCII printable
            } else if (0x20..0x7f).contains(&b) {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{:02x}", b)?;
            }
        }
        write!(f, "\"")?;
        Ok(())
    }
}
