/// provide simple serialization support for basic types
use super::bytes::*;

use crate::rust_kernel_linux_util::bindings::memcpy;
use crate::rust_kernel_linux_util::linux_kernel_module::c_types;

impl BytesMut {
    pub unsafe fn memcpy_serialize<T: Sized>(&mut self, v: &T) -> bool {
        let vlen = core::mem::size_of::<T>();
        if core::intrinsics::likely(vlen <= self.len()) {
            memcpy(
                (v as *const T as *mut T).cast::<c_types::c_void>(),
                self.ptr.cast::<c_types::c_void>(),
                vlen as u64,
            );
            return true;
        }
        false
    }

    pub unsafe fn memcpy_deserialize<T: Sized>(&mut self, target : &mut T) -> bool { 
        let vlen = core::mem::size_of::<T>();
        if core::intrinsics::likely(vlen <= self.len()) {
            memcpy(
                self.ptr.cast::<c_types::c_void>(),
                (target as *const T as *mut T).cast::<c_types::c_void>(),
                vlen as u64,
            );
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
