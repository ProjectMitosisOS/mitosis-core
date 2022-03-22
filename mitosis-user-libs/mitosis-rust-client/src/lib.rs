use std::os::unix::prelude::{AsRawFd, RawFd};

#[allow(unused_imports)]
pub(crate) use nix; 
pub(crate) use libc; 

/// The client ot issue MITOSIS system calls in rust
/// Must be created using MClientOptions
///
/// # Examples
///
/// ```no_run
/// 
/// use mitosis_rust_client::MClientOptions;
/// 
/// let client = MClientOptions::new().set_device_name("Cargo.toml".to_string()).open().unwrap();
#[allow(dead_code)]
pub struct MClient {
    fd: RawFd,
    file: std::fs::File,
}

/// The core system calls 
/// A process is identified globally a (u64, u64), 
/// where the first u64 is the container ID, and the second u64 is a user-provided key 
/// 
impl MClient {
    pub fn prepare(&mut self, _key : u64) { 
        unimplemented!();
    }

    // query the prepared results
    pub fn query(&mut self) -> Option<u64> {
        unimplemented!();
    }

    pub fn resume(&mut self, _id : u64, _key: u64) { 
        unimplemented!();
    }
}

impl MClient {
    pub(crate) fn new(f: std::fs::File) -> Self {
        Self {
            fd: f.as_raw_fd(),
            file: f,
        }
    }
}

/// Options to open a mitosis client that can use to call requests
///
/// # Examples
///
/// ```no_run
/// 
/// use mitosis_rust_client::MClientOptions;
/// 
/// let client = MClientOptions::new().set_device_name("Cargo.toml".to_string()).open().unwrap();
/// ```
pub struct MClientOptions {
    ioctl_device_name: String,
}

impl MClientOptions {
    pub fn new() -> Self {
        Self {
            ioctl_device_name: "".to_string(),
        }
    }

    pub fn set_device_name(&mut self, name: String) -> &mut Self {
        self.ioctl_device_name = name;
        self
    }

    pub fn open(&self) -> std::io::Result<MClient> {
        Ok(MClient::new(
            std::fs::File::options()
                .read(true)
                .open(self.ioctl_device_name.clone())?,
        ))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn client_option() {
        let _client = MClientOptions::new()
            .set_device_name("Cargo.toml".to_string())
            .open()
            .unwrap();
    }
}

pub mod macros;
