use super::header::*;
use crate::bytes::BytesMut;

pub struct ConnectStubFactory(usize);

impl ConnectStubFactory {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Generate the connect stub using a connect meta data (T)
    ///
    /// Return
    /// * if succeed, return the real message size
    ///
    pub fn generate<T: Sized>(self, meta: &T, msg: &mut BytesMut) -> core::option::Option<usize> {
        unsafe {
            Some(
                msg.memcpy_serialize_at(
                    0,
                    &MsgHeader::gen_connect_stub(self.0, core::mem::size_of_val(meta)),
                )? + msg.memcpy_serialize_at(core::mem::size_of::<MsgHeader>(), meta)?,
            )
        }
    }
}
