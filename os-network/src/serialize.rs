/// provide simple serialization support for basic types
use super::bytes::*;

impl BytesMut {
    /// Serialize a sized object through memory copy.
    ///
    pub unsafe fn memcpy_serialize<T: Sized>(&mut self, v: &T) -> core::option::Option<usize> {
        if core::intrinsics::likely(core::mem::size_of::<T>() <= self.len()) {
            core::ptr::copy_nonoverlapping(v, self.ptr as _, 1);
            return Some(core::mem::size_of::<T>());
        }
        None
    }

    /// Serialize a sized object through memory copy at an offset
    ///
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

    /// Deserialize a sized object through memory copy at an offset.
    ///
    pub unsafe fn memcpy_deserialize_at<T: Sized>(
        &self,
        off: usize,
        target: &mut T,
    ) -> core::option::Option<usize> {
        match self.truncate_header(off) {
            Some(bytes) => bytes.memcpy_deserialize(target),
            _ => None,
        }
    }

    /// Deserialize a sized object through memory copy.
    ///
    pub unsafe fn memcpy_deserialize<T: Sized>(
        &self,
        target: &mut T,
    ) -> core::option::Option<usize> {
        if core::intrinsics::likely(core::mem::size_of::<T>() <= self.len()) {
            core::ptr::copy_nonoverlapping(self.ptr as _, target, 1);
            return Some(core::mem::size_of::<T>());
        }
        None
    }
}

pub trait Serialize {
    /// Default implementation if serialize
    fn serialize(&self, bytes: &mut BytesMut) -> bool
    where
        Self: Sized,
    {
        unsafe { bytes.memcpy_serialize(self).is_some() }
    }

    /// Default implementation for stack data which only require copying
    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self>
    where
        Self: Sized,
    {
        let mut result: Self = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
        unsafe { bytes.memcpy_deserialize(&mut result)? };
        Some(result)
    }

    /// The size of buffer that is required for serialization
    fn serialization_buf_len(&self) -> usize
    where
        Self: Sized,
    {
        core::mem::size_of::<Self>()
    }
}
