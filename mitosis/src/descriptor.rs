#[allow(dead_code)]
pub struct Descriptor {
    regs: (),
    page_table: (),
    dct_key: (),
    // TODO: more need to be added
}

impl os_network::serialize::Serialize for Descriptor {
    fn serialize(&self, bytes: &mut BytesMut) -> bool {
        unimplemented!();
    }

    fn deserialize(bytes: &BytesMut) -> core::option::Option<Self> {
        unimplemented!();
    }
}
