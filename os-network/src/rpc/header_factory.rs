use super::header::*;
use crate::bytes::BytesMut;

#[allow(unused_imports)]
use KRdmaKit::rust_kernel_rdma_base::linux_kernel_module;

pub struct ConnectStubFactory(usize);

impl ConnectStubFactory {
    #[inline]
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Generate the connect stub using a connect meta data (T)
    ///
    /// Return
    /// * if succeed, return the real message size
    ///
    #[inline]
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

pub struct ReplyStubFactory {
    status: ReplyStatus,
    payload: usize,
}

impl ReplyStubFactory {
    #[inline]
    pub fn new(status: ReplyStatus, sz: usize) -> Self {
        Self {
            status: status,
            payload: sz,
        }
    }

    #[inline]
    pub fn get_payload(&self) -> usize { 
        self.payload
    }

    #[inline]
    pub fn generate(self, msg: &mut BytesMut) -> core::option::Option<usize> {
        unsafe { msg.memcpy_serialize_at(0, &MsgHeader::gen_reply_stub(self.status, self.payload)) }
    }
}
