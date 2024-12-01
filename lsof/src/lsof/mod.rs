use crate::lsof::error::Error;
use crate::lsof::unix::Unix;
use chrono::Local;
use std::env::consts;
use std::fmt;

pub mod error;
pub mod unix;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum FileType {
    REGULAR,
    NETWORK,
    ALL,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct OpenFile {
    pub command: String, // Command
    pub pid: u32,        // Process ID
    pub uid: i16,        // User ID
    pub fd: String,      // File Descriptor
    pub _type: String,   // Column type
    pub device: String,  // Device
    pub size: u64,       // Size
    pub node: String,    // Node
    pub name: String,    // Name
    pub created_at: i64, // Timestamp command execution
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FileType::REGULAR => "regular",
            FileType::NETWORK => "network",
            FileType::ALL => "all",
        };
        write!(f, "{}", s)
    }
}

impl OpenFile {
    pub fn new(pid: u32, uid: i16, command: &str) -> Self {
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
            created_at: Local::now().timestamp_millis(),
        }
    }
}

pub trait Lsof {
    fn exec(file_type: &FileType) -> Result<Vec<OpenFile>, Error>;
}

pub fn lsof(file_type: &FileType) -> Result<Vec<OpenFile>, Error> {
    if ["linux", "macos"].contains(&consts::OS) {
        Unix::exec(file_type)
    } else {
        Err(Error::Unimplemented {
            os: consts::OS.to_string(),
            arch: consts::ARCH.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::lsof::{lsof, FileType};

    #[test]
    fn test_lsof() {
        let regular = lsof(&FileType::REGULAR).unwrap();
        let network = lsof(&FileType::NETWORK).unwrap();
        let all = lsof(&FileType::ALL).unwrap();
        assert!(!regular.is_empty());
        assert!(!network.is_empty());
        assert!(!all.is_empty());
        assert!(regular.len() > network.len());
        assert!(all.len() > regular.len());
    }
}
