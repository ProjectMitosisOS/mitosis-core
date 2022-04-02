#![no_std]

pub type IoctlCmdType = u32;

/// a simple call cmd to test the work of system calls 
pub const CALL_NIL : IoctlCmdType = 0;

/// Establish a UD connection from the remote end
pub const CALL_CONNECT : IoctlCmdType = 3;

/// Prepare the caller process's state to a shadow process at the caller machine
pub const CALL_PREPARE : IoctlCmdType = 4;

/// Resume from a local image
pub const CALL_RESUME_LOCAL : IoctlCmdType = 5;

/// Resume from a remote image
pub const CALL_RESUME_REMOTE : IoctlCmdType = 5;

