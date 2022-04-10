use crate::{ioctl_write, ioctl_test};

ioctl_write!(mitosis_syscall_nil, mitosis_protocol::CALL_NIL as _, usize);
ioctl_write!(mitosis_syscall_prepare, mitosis_protocol::CALL_PREPARE as _, u64);

ioctl_test!(mitosis_test,  usize);
