use crate::lsof::error::Error;
use crate::lsof::unix::Unix;
use chrono::Local;
use std::env::consts;

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
    pub created_at: i64, // Timestamp command execution
}

impl OpenFile {
    pub fn new(pid: u32, uid: u32, command: &str) -> Self {
        OpenFile {
            command: command.to_string(),
            pid,
            uid,
            fd: "".to_string(),
            _type: "".to_string(),
            device: "".to_string(),
            size: 0,
            node: "".to_string(),
            name: "".to_string(),
            created_at: Local::now().timestamp(),
        }
    }
}

pub trait Lsof {
    fn exec() -> Result<Vec<OpenFile>, Error>;
}

pub fn lsof() -> Result<Vec<OpenFile>, Error> {
    if ["linux", "macos"].contains(&consts::OS) {
        Unix::exec()
    } else {
        Err(Error::Unimplemented {
            os: consts::OS.to_string(),
            arch: consts::ARCH.to_string(),
        })
    }
}
