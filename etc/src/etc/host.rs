use crate::etc::error::Error;
use crate::etc::EtcReader;
use std::fs;

const FILE_PATH: &str = "/etc/hosts";

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct Host {
    pub name: String,
    pub address: String,
}

impl EtcReader<Host> for Host {
    fn read_etc_file(path: Option<&str>) -> Result<Vec<Host>, Error> {
        let mut host_buffer: Vec<Host> = vec![];
        let path = if let Some(path) = path {
            path
        } else {
            FILE_PATH
        };
        let contents = fs::read_to_string(path)?;
        for line in contents.lines() {
            for host in parse_host_row(line) {
                if !host_buffer.contains(&host) {
                    host_buffer.push(host)
                }
            }
        }
        Ok(host_buffer)
    }
}

fn parse_host_row(row: &str) -> Vec<Host> {
    let mut host_buffer = vec![];
    if !row.starts_with('#') {
        let fields: Vec<&str> = row.split_whitespace().collect();
        if fields.len() >= 2 {
            for field in &fields[1..] {
                host_buffer.push(Host {
                    name: field.to_string(),
                    address: fields[0].to_string(),
                })
            }
        }
    }
    host_buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_etc_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "127.0.0.1\tlocalhost").unwrap();
        writeln!(temp_file, "255.255.255.255\tbroadcasthost").unwrap();
        writeln!(temp_file, "# Comment line").unwrap();
        writeln!(
            temp_file,
            "172.22.17.10\thello.database.windows.net other.net"
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap();

        let hosts = Host::read_etc_file(Some(path)).unwrap();

        assert_eq!(hosts.len(), 4);
        assert!(hosts.contains(&Host {
            name: "hello.database.windows.net".to_string(),
            address: "172.22.17.10".to_string(),
        }));
        assert!(hosts.contains(&Host {
            name: "other.net".to_string(),
            address: "172.22.17.10".to_string(),
        }));
        assert!(hosts.contains(&Host {
            name: "localhost".to_string(),
            address: "127.0.0.1".to_string(),
        }));
        assert!(hosts.contains(&Host {
            name: "broadcasthost".to_string(),
            address: "255.255.255.255".to_string(),
        }));
    }

    #[test]
    fn test_read_etc_file_with_duplicates() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "127.0.0.1\tlocalhost").unwrap();
        writeln!(temp_file, "127.0.0.1\tlocalhost").unwrap();
        writeln!(temp_file, "# Comment line").unwrap();
        writeln!(temp_file, "172.22.17.10\thello.database.windows.net").unwrap();
        let path = temp_file.path().to_str().unwrap();

        let hosts = Host::read_etc_file(Some(path)).unwrap();

        assert_eq!(hosts.len(), 2);
        assert!(hosts.contains(&Host {
            name: "hello.database.windows.net".to_string(),
            address: "172.22.17.10".to_string(),
        }));
        assert!(hosts.contains(&Host {
            name: "localhost".to_string(),
            address: "127.0.0.1".to_string(),
        }));
    }

    #[test]
    fn test_read_etc_file_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let hosts = Host::read_etc_file(Some(temp_file.path().to_str().unwrap())).unwrap();
        assert_eq!(hosts.len(), 0);
    }

    #[test]
    fn test_read_etc_file_no_hosts() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "# Another comment").unwrap();

        let hosts = Host::read_etc_file(Some(temp_file.path().to_str().unwrap())).unwrap();

        assert_eq!(hosts.len(), 0);
    }

    #[test]
    fn test_parse_host_row_simple() {
        let host = parse_host_row("127.0.0.1\tlocalhost");
        assert_eq!(host.len(), 1);
        assert_eq!(
            host[0],
            Host {
                name: "localhost".to_string(),
                address: "127.0.0.1".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_host_row_multiple() {
        let host = parse_host_row("127.0.0.1\tlocalhost loopback");
        assert_eq!(host.len(), 2);
        assert_eq!(
            host[0],
            Host {
                name: "localhost".to_string(),
                address: "127.0.0.1".to_string(),
            }
        );
        assert_eq!(
            host[1],
            Host {
                name: "loopback".to_string(),
                address: "127.0.0.1".to_string(),
            }
        );
    }
}
