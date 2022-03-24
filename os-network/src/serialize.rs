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
    fn serialize(&self, bytes: &mut BytesMut) -> bool;
    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self>;
}
