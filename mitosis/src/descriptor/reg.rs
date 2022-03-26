use os_network::bytes::BytesMut;
pub struct RegDescriptor {
    others: crate::bindings::StackRegisters,
    fs: u64,
    gs: u64,
}

impl os_network::serialize::Serialize for RegDescriptor {

    /// TODO: before writing the serialization & deserialization, 
    /// need to first illustrate the buffer format in the comments, 
    /// e.g., 
    /// ```
    /// |fs <-u64-> | gs <-u64-> | ....
    /// ```
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        unimplemented!();
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        unimplemented!();
    }
}