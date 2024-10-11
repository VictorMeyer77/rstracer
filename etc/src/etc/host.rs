use crate::etc::error::Error;
use crate::etc::EtcReader;
use std::fs;
use std::process::Command;

const FILE_PATH: &str = "/etc/hosts";

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Host {
    pub name: String,
    pub address: String,
}

impl EtcReader<Host> for Host {
    fn read_etc_file(path: Option<&str>) -> Result<Vec<Host>, Error> {
        let mut host_buffer: Vec<Host> = vec![get_host_row()?];
        let path = if let Some(path) = path {
            path
        } else {
            FILE_PATH
        };
        let contents = fs::read_to_string(path)?;
        for line in contents.lines() {
            if !line.starts_with('#') {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 2 {
                    host_buffer.push(Host {
                        name: fields[1].to_string(),
                        address: fields[0].to_string(),
                    })
                }
            }
        }

        Ok(host_buffer)
    }
}

fn get_host_row() -> Result<Host, Error> {
    let hostname = String::from_utf8_lossy(&Command::new("hostname").output()?.stdout)
        .trim()
        .to_string();
    let address = String::from_utf8_lossy(
        &Command::new("dig")
            .args(["+short", &hostname])
            .output()?
            .stdout,
    )
    .trim()
    .to_string();
    Ok(Host {
        name: hostname,
        address,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_etc_file_with_mock_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "127.0.0.1\tlocalhost").unwrap();
        writeln!(temp_file, "255.255.255.255\tbroadcasthost").unwrap();
        writeln!(temp_file, "# Comment line").unwrap();
        writeln!(temp_file, "172.22.17.10\thello.database.windows.net").unwrap();
        let path = temp_file.path().to_str().unwrap();

        let hosts = Host::read_etc_file(Some(path)).unwrap();

        assert_eq!(hosts.len(), 4);
        assert_eq!(
            hosts[1],
            Host {
                name: "localhost".to_string(),
                address: "127.0.0.1".to_string(),
            }
        );
        assert_eq!(
            hosts[2],
            Host {
                name: "broadcasthost".to_string(),
                address: "255.255.255.255".to_string(),
            }
        );
        assert_eq!(
            hosts[3],
            Host {
                name: "hello.database.windows.net".to_string(),
                address: "172.22.17.10".to_string(),
            }
        );
    }

    #[test]
    fn test_read_etc_file_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let hosts = Host::read_etc_file(Some(temp_file.path().to_str().unwrap())).unwrap();
        assert_eq!(hosts.len(), 1);
    }

    #[test]
    fn test_read_etc_file_no_hosts() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "# Another comment").unwrap();

        let hosts = Host::read_etc_file(Some(temp_file.path().to_str().unwrap())).unwrap();

        assert_eq!(hosts.len(), 1);
    }

    #[test]
    fn test_get_host_row() {
        let result = get_host_row();
        assert!(result.is_ok());
        let host = result.unwrap();
        assert!(!host.name.is_empty());
    }
}
