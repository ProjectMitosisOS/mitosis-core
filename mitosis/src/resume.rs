mod fallback;
pub use fallback::*;

mod remote_paging;
pub use remote_paging::*;

use os_network::rdma::RawGID;

#[allow(unused_imports)]
use crate::linux_kernel_module;
use crate::descriptors::{RDMADescriptor};
use crate::get_descriptor_pool_mut;
use crate::linux_kernel_module::c_types::c_long;

/// Fork prepare in parent's process.
/// Generate one shadow container and record its descriptor
///
/// Param handler_id: Prepare key
///
/// Param nic_idx:    Used for assign the RNIC0 or RNIC1
#[inline]
#[warn(dead_code)]
pub fn fork_prepare_impl(handler_id: usize, nic_id: usize) -> c_long {
    let context = unsafe {
        crate::get_rpc_caller_pool_ref()
            .get_caller_context(nic_id)
            .unwrap()
    };
    let des_pool = unsafe { get_descriptor_pool_mut() };
    let raw_gid = RawGID::new(context.get_gid_as_string());
    if raw_gid.is_some() {
        des_pool.put_current_descriptor(handler_id, RDMADescriptor {
            gid: RawGID::new(context.get_gid_as_string()).unwrap(),
            service_id: crate::rdma_context::SERVICE_ID_BASE + nic_id as u64,
            rkey: 64,
        });
    }
    0
}
