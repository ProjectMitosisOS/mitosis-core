/// The datagram caller is a caller implemented in RDMA's unreliable datagram (UD).
/// It assumes that the size of the message is within the
/// datagram packet.
/// In RDMA's UD case, maximum supported message is MTU
pub struct Caller {}