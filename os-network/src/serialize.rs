/// provide simple serialization support for basic types
use super::bytes::*;

impl BytesMut {
    pub unsafe fn memcpy_serialize<T: Sized>(&mut self, v: &T) -> core::option::Option<usize> {
        if core::intrinsics::likely(core::mem::size_of::<T>() <= self.len()) {
            core::ptr::copy_nonoverlapping(v, self.ptr as _, 1); 
            return Some(core::mem::size_of::<T>());
        }
        None
    }

    pub unsafe fn memcpy_serialize_at<T: Sized>(
        &mut self,
        off: usize,
        v: &T,
    ) -> core::option::Option<usize> {
        match self.truncate_header(off) {
            Some(mut bytes) => bytes.memcpy_serialize(v),
            _ => None,
        }
    }

    pub unsafe fn memcpy_deserialize<T: Sized>(&self, target: &mut T) -> core::option::Option<usize>  {
        if core::intrinsics::likely(core::mem::size_of::<T>() <= self.len()) {
            core::ptr::copy_nonoverlapping(self.ptr as _, target, 1);
            return Some(core::mem::size_of::<T>());
        }        
        None
    }
}

pub trait Serialize
where
    Self: Sized,
{
    /// Default implementation for stack data which only require copying
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        let src = unsafe {
            BytesMut::from_raw(self as *const _ as *mut u8, core::mem::size_of::<Self>())
        };
        bytes.copy(&src, 0)
    }

    /// Default implementation for stack data which only require copying
    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let size = core::mem::size_of::<Self>();
        if bytes.len() != size {
            return None;
        }
        let result: Self = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
        let mut dst = unsafe {
            BytesMut::from_raw(&result as *const _ as *mut u8, core::mem::size_of::<Self>())
        };
        if dst.copy(bytes, 0) {
            Some(result)
        } else {
            None
        }
    }

    /// Default implementation for stack data which only require the size on the stack
    fn serialization_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}
