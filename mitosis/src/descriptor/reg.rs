#[derive(Default)]
pub struct RegDescriptor {
    pub others: crate::bindings::StackRegisters,
    pub fs: u64,
    pub gs: u64,
}

impl os_network::serialize::Serialize for RegDescriptor {}
