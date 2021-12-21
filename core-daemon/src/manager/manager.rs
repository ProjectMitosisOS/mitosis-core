pub struct MitosisManager {
    // TODO
}

pub(crate) static mut MY_VM_OP: crate::bindings::vm_operations_struct = unsafe {
    core::mem::transmute([0u8; core::mem::size_of::<crate::bindings::vm_operations_struct>()])
};
