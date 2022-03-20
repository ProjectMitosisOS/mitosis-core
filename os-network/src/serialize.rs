/// provide simple serialization support for basic types
use super::bytes::*;

impl BytesMut {
    pub unsafe fn memcpy_serialize<T: Sized>(&mut self, v: &T) -> bool {
        if core::intrinsics::likely(core::mem::size_of::<T>() <= self.len()) {
            unsafe { core::ptr::copy_nonoverlapping(v, self.ptr as _, 1) };
            return true;
        }
        false
    }

    pub unsafe fn memcpy_deserialize<T: Sized>(&mut self, target: &mut T) -> bool {
        if core::intrinsics::likely(core::mem::size_of::<T>() <= self.len()) {
            unsafe { core::ptr::copy_nonoverlapping(self.ptr as _, target, 1) };
            return true;
        }
        false
    }
}

pub trait Serialize
where
    Self: Sized,
{
    fn serialize(&self, bytes: &mut BytesMut) -> bool;
    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self>;
}
