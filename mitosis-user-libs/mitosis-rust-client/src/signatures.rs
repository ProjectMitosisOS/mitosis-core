use crate::{ioctl_write, ioctl_test};

ioctl_write!(mitosis_syscall_nil, mitosis_protocol::CALL_NIL as _, usize); 

ioctl_test!(mitosis_test,  usize);
