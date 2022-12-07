use core::fmt::Write;
use os_network::bytes::BytesMut;
use os_network::serialize::Serialize;

#[derive(Debug)]
#[repr(usize)]
pub enum RPCId {
    // for test only
    Nil = 1,
    // for test only
    Echo = 2,
    // Resume fork by fetching remote descriptor
    Query = 3,
}

pub(crate) fn handle_nil(_input: &BytesMut, _output: &mut BytesMut) -> usize {
    64
}

pub(crate) fn handle_echo(input: &BytesMut, output: &mut BytesMut) -> usize {
    crate::log::info!("echo callback {:?}", input);
    write!(output, "Hello from MITOSIS").unwrap();
    64
}

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct DescriptorLookupReply {
    pub(crate) pa: u64,
    pub(crate) sz: usize,
    pub(crate) ready: bool,

    // for remote dct access
    pub(crate) rkey: u32,
    pub(crate) lid: u32,
    pub(crate) gid: rust_kernel_rdma_base::ib_gid,
    pub(crate) dct_num: u32,
    pub(crate) dc_key: u64,

    #[cfg(feature = "use_rc")]
    // for rc connection
    pub(crate) rc_rkey: u32,
}

impl os_network::serialize::Serialize for DescriptorLookupReply {}

pub(crate) fn handle_descriptor_addr_lookup(input: &BytesMut, output: &mut BytesMut) -> usize {
    let mut key: usize = 0;
    unsafe { input.memcpy_deserialize(&mut key) };

    let process_service = unsafe { crate::get_sps_mut() };
    let buf = process_service.query_descriptor_buf(key);

    if buf.is_none() {
        crate::log::error!("empty addr, key:{}!", key);

        // send an error reply
        return 0; // a null reply indicate that the we don't have the key
    }

    let dc_target_idx = unsafe { crate::bindings::pmem_get_current_cpu()  as usize % (crate::dc_target::get_ref().len()) };
    let dc_target = unsafe { crate::dc_target::get_ref().get(dc_target_idx).unwrap() };

    let meta = unsafe { crate::dc_target_meta::get_ref().get(dc_target_idx).unwrap() };
    let (lid, gid) = (meta.lid, meta.gid);

    #[cfg(feature = "use_rc")]
    let rc_server_idx = unsafe { crate::bindings::pmem_get_current_cpu()  as usize % (crate::rc_cm_service::get_ref().len()) };
    #[cfg(feature = "use_rc")]
    let rc_server = unsafe { crate::rc_service::get_ref().get(rc_server_idx).unwrap() };
    
    let reply = match buf {
        Some((addr, len)) => {
            DescriptorLookupReply {
                pa: addr.get_pa(),
                sz: len,
                ready: true,

                rkey: dc_target.ctx().rkey(),
                lid: lid as u32,
                gid,
                dct_num: dc_target.dct_num(),
                dc_key: dc_target.dc_key(),

                #[cfg(feature = "use_rc")]
                rc_rkey: rc_server.ctx().rkey(),
            }
        }
        None => {
            crate::log::error!("Failed to find the handner with id: {}!", key);
            DescriptorLookupReply {
                pa: 0,
                sz: 0,
                ready: false,

                rkey: 0,
                lid: 0,
                gid: Default::default(),
                dct_num: 0,
                dc_key: 0,
                
                #[cfg(feature = "use_rc")]
                rc_rkey: 0,
            }
        }
    };

    reply.serialize(output);
    reply.serialization_buf_len()
}

