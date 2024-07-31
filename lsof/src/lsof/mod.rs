use crate::lsof::error::Error;
use crate::lsof::unix::Unix;
use std::env::consts;
use std::process::Output;

pub mod error;
pub mod unix;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct OpenFile {
    pub command: String, // Command
    pub pid: u32,        // Process ID
    pub uid: u32,        // User ID
    pub fd: String,      // File Descriptor
    pub _type: String,   // Column type
    pub device: String,  // Device
    pub size: u32,       // Size
    pub node: String,    // Node
    pub name: String,    // Name
}

pub trait Lsof {
    fn os_command() -> Result<Output, Error>;
    fn parse_output(output: &str) -> Result<Vec<OpenFile>, Error>;
    fn exec() -> Result<Vec<OpenFile>, Error> {
        let output = Self::os_command()?;
        Self::parse_output(&String::from_utf8_lossy(&output.stdout))
    }
}

pub fn lsof() -> Result<Vec<OpenFile>, Error> {
    if ["linux", "macos", "android", "ios"].contains(&consts::OS) {
        Unix::exec()
    } else {
        Err(Error::Unimplemented {
            os: consts::OS.to_string(),
            arch: consts::ARCH.to_string(),
        })
    }
}
