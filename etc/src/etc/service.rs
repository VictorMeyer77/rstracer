use crate::etc::error::Error;
use crate::etc::EtcReader;
use regex::Regex;
use std::fs;

const FILE_PATH: &str = "/etc/services";
const ROW_REGEX: &str = r"^([a-zA-Z0-9-]+)\s+(\d{1,5})\/([a-zA-Z0-9-]+)";

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Service {
    name: String,
    port: u16,
    protocol: String,
}

impl EtcReader<Service> for Service {
    fn read_etc_file(path: Option<&str>) -> Result<Vec<Service>, Error> {
        let mut services_buffer: Vec<Service> = vec![];
        let regex = Regex::new(ROW_REGEX)?;
        let path = if let Some(path) = path {
            path
        } else {
            FILE_PATH
        };
        let contents = fs::read_to_string(path)?;
        for line in contents.lines() {
            if let Some(captures) = regex.captures(line) {
                services_buffer.push(Service {
                    name: captures[1].to_string(),
                    port: captures[2].parse()?,
                    protocol: captures[3].to_string(),
                })
            }
        }
        Ok(services_buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_etc_file_valid_content() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "http           80/tcp\nhttps          443/tcp\nntp            123/udp"
        ).unwrap();
        let services = Service::read_etc_file(temp_file.path().to_str()).unwrap();
        let expected = vec![
            Service {
                name: "http".to_string(),
                port: 80,
                protocol: "tcp".to_string(),
            },
            Service {
                name: "https".to_string(),
                port: 443,
                protocol: "tcp".to_string(),
            },
            Service {
                name: "ntp".to_string(),
                port: 123,
                protocol: "udp".to_string(),
            },
        ];
        assert_eq!(services, expected);
    }

    #[test]
    fn test_read_etc_file_invalid_content() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid_entry\nhttp           eighty/tcp").unwrap();
        let services = Service::read_etc_file(temp_file.path().to_str()).unwrap();
        assert_eq!(services.len(), 0);
    }

    #[test]
    fn test_read_etc_file_empty_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let services = Service::read_etc_file(temp_file.path().to_str()).unwrap();
        assert_eq!(services.len(), 0);
    }
}
