#[allow(unused_imports)]
use crate::linux_kernel_module;
use os_network::bytes::BytesMut;

pub trait IntValue = PartialEq + Eq + Sized + Default + Copy + Clone;

#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
#[repr(packed, C)]
pub struct Pair<K: IntValue, V: IntValue> {
    k: K,
    v: V,
}

impl<K: IntValue, V: IntValue> Pair<K, V> {
    pub fn new(k: K, v: V) -> Self {
        Self { k, v }
    }

    #[inline]
    pub fn get_0(&self)->&K {
        &self.k
    }

    #[inline]
    pub fn get_1(&self) -> &V{
        &self.v
    }

    #[inline]
    pub fn get_pair(&self) -> (&K,&V) {
        (&self.k, &self.v)
    }

}

impl<K: IntValue, V: IntValue> os_network::serialize::Serialize for Pair<K, V> {
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        if bytes.len() < self.serialization_buf_len() {
            crate::log::error!(
                "failed to serialize: buffer space not enough. Need {}, actual {}",
                self.serialization_buf_len(),
                bytes.len()
            );
            return false;
        }
        unsafe { core::ptr::write_unaligned(bytes.get_ptr() as *mut Self, *self) };
        true
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        let cur = unsafe { bytes.truncate_header(0)? };
        let mut v: Pair<K, V> = Default::default();
        unsafe { cur.memcpy_deserialize_at(0, &mut v)? };

        unsafe { Some(core::ptr::read_unaligned(cur.get_ptr() as *const Self)) }
    }

    fn serialization_buf_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}
