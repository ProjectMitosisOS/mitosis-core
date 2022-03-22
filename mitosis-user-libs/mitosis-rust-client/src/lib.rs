use std::os::unix::prelude::{AsRawFd, RawFd};
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
    fn ioctrl() {
        let _client = MClientOptions::new()
            .set_device_name("Cargo.toml".to_string())
            .open()
            .unwrap();
    }
}
