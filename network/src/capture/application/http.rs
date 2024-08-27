use crate::error::Error;

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
    Connect,
    Trace,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpType {
    Request,
    Response,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpVersion {
    V0_9,
    V1_0,
    V1_1,
    V2,
    V3,
}

#[derive(Debug, Clone)]
pub struct Http {
    pub instruction: HttpInstruction,
    pub headers: HttpHeader,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct HttpInstruction {
    pub _type: HttpType,
    pub method: Option<HttpMethod>,
    pub uri: Option<String>,
    pub version: HttpVersion,
    pub status_code: Option<u16>,
    pub status_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HttpHeader {
    pub headers: HashMap<String, String>,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HttpMethod::Get => "get",
                HttpMethod::Post => "post",
                HttpMethod::Put => "put",
                HttpMethod::Delete => "delete",
                HttpMethod::Patch => "patch",
                HttpMethod::Options => "options",
                HttpMethod::Head => "head",
                HttpMethod::Connect => "connect",
                HttpMethod::Trace => "trace",
            }
        )
    }
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<HttpMethod, Self::Err> {
        match s.to_lowercase().as_str() {
            "get" => Ok(HttpMethod::Get),
            "post" => Ok(HttpMethod::Post),
            "put" => Ok(HttpMethod::Put),
            "delete" => Ok(HttpMethod::Delete),
            "patch" => Ok(HttpMethod::Patch),
            "options" => Ok(HttpMethod::Options),
            "head" => Ok(HttpMethod::Head),
            "connect" => Ok(HttpMethod::Connect),
            "trace" => Ok(HttpMethod::Trace),
            _ => Err(()),
        }
    }
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HttpVersion::V0_9 => "HTTP/0.9",
                HttpVersion::V1_0 => "HTTP/1.0",
                HttpVersion::V1_1 => "HTTP/1.1",
                HttpVersion::V2 => "HTTP/2",
                HttpVersion::V3 => "HTTP/3",
            }
        )
    }
}

impl FromStr for HttpVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<HttpVersion, Self::Err> {
        match s.to_lowercase().as_str() {
            "http/0.9" => Ok(HttpVersion::V0_9),
            "http/1.0" => Ok(HttpVersion::V1_0),
            "http/1.1" => Ok(HttpVersion::V1_1),
            "http/2" => Ok(HttpVersion::V2),
            "http/3" => Ok(HttpVersion::V3),
            _ => Err(()),
        }
    }
}

impl Http {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if !bytes.is_empty() {
            let utf_packet = String::from_utf8(bytes.to_vec()).map_err(|_| Error::NomParsing)?;
            let rows: Vec<&str> = utf_packet.split("\r\n").collect();
            if let Some(empty_line_index) = rows.iter().position(|&x| x.is_empty()) {
                return Ok(Http {
                    instruction: HttpInstruction::from_str(rows[0])?,
                    headers: HttpHeader::from_lines(&rows[1..empty_line_index])?,
                    body: rows[(empty_line_index + 1)..].join("\r\n"),
                });
            }
        }
        Err(Error::NomParsing)
    }
}

impl HttpInstruction {
    fn from_str(row: &str) -> Result<HttpInstruction, Error> {
        if let Ok(instruction) = Self::request_from_str(row) {
            Ok(instruction)
        } else {
            Self::response_from_str(row)
        }
    }
    fn request(method: HttpMethod, uri: String, version: HttpVersion) -> Self {
        HttpInstruction {
            _type: HttpType::Request,
            method: Some(method),
            uri: Some(uri),
            version,
            status_code: None,
            status_text: None,
        }
    }

    fn response(version: HttpVersion, status_code: u16, status_text: String) -> Self {
        HttpInstruction {
            _type: HttpType::Response,
            method: None,
            uri: None,
            version,
            status_code: Some(status_code),
            status_text: Some(status_text),
        }
    }

    fn request_from_str(row: &str) -> Result<HttpInstruction, Error> {
        let fields: Vec<&str> = row.split_whitespace().collect();
        let method = HttpMethod::from_str(fields[0]).map_err(|_| Error::NomParsing)?;
        let version = HttpVersion::from_str(fields[2]).map_err(|_| Error::NomParsing)?;
        Ok(HttpInstruction::request(
            method,
            fields[1].to_string(),
            version,
        ))
    }

    fn response_from_str(row: &str) -> Result<HttpInstruction, Error> {
        let fields: Vec<&str> = row.split_whitespace().collect();
        let version = HttpVersion::from_str(fields[0]).map_err(|_| Error::NomParsing)?;
        let status_code = fields[1].parse::<u16>().map_err(|_| Error::NomParsing)?;
        Ok(HttpInstruction::response(
            version,
            status_code,
            fields[2..].join(" "),
        ))
    }
}

impl HttpHeader {
    fn from_lines(lines: &[&str]) -> Result<HttpHeader, Error> {
        let headers: Vec<Vec<&str>> = lines
            .iter()
            .map(|line| line.splitn(2, ':').map(|chunk| chunk.trim()).collect())
            .collect();

        if headers.iter().filter(|header| header.len() != 2).count() > 0 {
            Err(Error::NomParsing)
        } else {
            Ok(HttpHeader {
                headers: headers
                    .iter()
                    .map(|header| (header[0].to_string(), header[1].to_string()))
                    .collect(),
            })
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn create_http_packet() -> Vec<u8> {
        let content = "name=ChatGPT&language=Rust";
        format!(
            "POST /submit HTTP/1.1\r\n\
        Host: example.com\r\n\
        Content-Type: application/x-www-form-urlencoded\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\r\n\
        {}",
            content.len(),
            content
        )
        .as_bytes()
        .to_vec()
    }

    #[test]
    fn test_http_instruction_request_from_str() {
        let instruction = "GET /websiteos/example.com HTTP/1.1";
        let instruction = HttpInstruction::from_str(instruction).unwrap();

        assert_eq!(instruction._type, HttpType::Request);
        assert_eq!(instruction.method.unwrap(), HttpMethod::Get);
        assert_eq!(instruction.uri.unwrap(), "/websiteos/example.com");
        assert_eq!(instruction.version, HttpVersion::V1_1);
        assert!(instruction.status_code.is_none());
        assert!(instruction.status_text.is_none());
    }

    #[test]
    fn test_http_instruction_response_from_str() {
        let instruction = "HTTP/2 304 Not Modified";
        let instruction = HttpInstruction::from_str(instruction).unwrap();

        assert_eq!(instruction._type, HttpType::Response);
        assert!(instruction.method.is_none());
        assert!(instruction.uri.is_none());
        assert_eq!(instruction.version, HttpVersion::V2);
        assert_eq!(instruction.status_code.unwrap(), 304);
        assert_eq!(instruction.status_text.unwrap(), "Not Modified");
    }

    #[test]
    fn test_http_instruction_from_str_invalid() {
        let instruction = "invalid HTTP/1.1 304 Not Modified";
        let instruction = HttpInstruction::from_str(instruction);
        assert!(instruction.is_err());
    }

    #[test]
    fn test_http_header_from_lines() {
        let header = vec![
            "Host: example.com",
            "Content-Type: application/x-www-form-urlencoded",
            "Content-Length: 26",
            "Connection: close",
        ];
        let header = HttpHeader::from_lines(&header);

        let headers = header.unwrap().headers;
        assert_eq!(headers.get("Host").unwrap(), "example.com");
        assert_eq!(
            headers.get("Content-Type").unwrap(),
            "application/x-www-form-urlencoded"
        );
        assert_eq!(headers.get("Content-Length").unwrap(), "26");
        assert_eq!(headers.get("Connection").unwrap(), "close");
    }

    #[test]
    fn test_http_header_from_lines_invalid() {
        let header = vec!["Host example.com"];
        let header = HttpHeader::from_lines(&header);

        assert!(header.is_err())
    }

    #[test]
    fn test_http_from_bytes() {
        let packet = create_http_packet();

        let http = Http::from_bytes(&packet).unwrap();

        let instruction = http.instruction;
        assert_eq!(instruction.method.unwrap(), HttpMethod::Post);
        let headers = http.headers;
        assert_eq!(headers.headers.get("Host").unwrap(), "example.com");
        assert_eq!(http.body, "name=ChatGPT&language=Rust");
    }

    #[test]
    fn test_http_from_bytes_invalid() {
        let packet = "POST /submit HTTP/1.1\r\n\
        Host: example.com\r\n\
        Connection: close"
            .as_bytes();

        let http = Http::from_bytes(packet);
        assert!(http.is_err());
    }
}
