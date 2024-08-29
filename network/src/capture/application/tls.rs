use crate::error::Error;
use nom::bytes::complete::take;
use nom::number::complete::be_u16;
use nom::IResult;

#[derive(Debug, Clone, PartialEq)]
pub enum TlsContentType {
    ChangeCipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    ApplicationData = 23,
    Heartbeat = 24,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TlsVersion {
    SSL20 = 2,
    SSL30 = 768,
    TLS10 = 769,
    TLS11 = 770,
    TLS12 = 771,
    TLS13 = 772,
}

#[derive(Debug, Clone)]
pub struct Tls {
    pub content_type: TlsContentType,
    pub version: TlsVersion,
    pub length: u16,
    pub payload: Vec<u8>,
}

impl TlsContentType {
    fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x14 => Some(TlsContentType::ChangeCipherSpec),
            0x15 => Some(TlsContentType::Alert),
            0x16 => Some(TlsContentType::Handshake),
            0x17 => Some(TlsContentType::ApplicationData),
            0x18 => Some(TlsContentType::Heartbeat),
            _ => None,
        }
    }
}

impl TlsVersion {
    fn from_u16(version: u16) -> Option<Self> {
        match version {
            2 => Some(TlsVersion::SSL20),
            768 => Some(TlsVersion::SSL30),
            769 => Some(TlsVersion::TLS10),
            770 => Some(TlsVersion::TLS11),
            771 => Some(TlsVersion::TLS12),
            772 => Some(TlsVersion::TLS13),
            _ => None,
        }
    }
}

impl Tls {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if let Ok((bytes, (content_type, version, length))) = Self::parse_header_bytes(bytes) {
            Ok(Self::new(content_type, version, length, bytes)?)
        } else {
            Err(Error::ApplicationParsing)
        }
    }

    fn new(content_type: u8, version: u16, length: u16, payload: &[u8]) -> Result<Self, Error> {
        if let Some(content_type) = TlsContentType::from_u8(content_type) {
            if let Some(version) = TlsVersion::from_u16(version) {
                return Ok(Tls {
                    content_type,
                    version,
                    length,
                    payload: payload.to_vec(),
                });
            }
        }
        Err(Error::ApplicationParsing)
    }

    fn parse_header_bytes(bytes: &[u8]) -> IResult<&[u8], (u8, u16, u16)> {
        let (bytes, ct_byte) = take(1usize)(bytes)?;
        let (bytes, version) = be_u16(bytes)?;
        let (bytes, length) = be_u16(bytes)?;
        Ok((bytes, (ct_byte[0], version, length)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_content_type_from_u8() {
        assert_eq!(
            TlsContentType::from_u8(0x14),
            Some(TlsContentType::ChangeCipherSpec)
        );
        assert_eq!(TlsContentType::from_u8(0x15), Some(TlsContentType::Alert));
        assert_eq!(
            TlsContentType::from_u8(0x16),
            Some(TlsContentType::Handshake)
        );
        assert_eq!(
            TlsContentType::from_u8(0x17),
            Some(TlsContentType::ApplicationData)
        );
        assert_eq!(
            TlsContentType::from_u8(0x18),
            Some(TlsContentType::Heartbeat)
        );
        assert_eq!(TlsContentType::from_u8(0x19), None);
    }

    #[test]
    fn test_tls_version_from_u16() {
        assert_eq!(TlsVersion::from_u16(2), Some(TlsVersion::SSL20));
        assert_eq!(TlsVersion::from_u16(768), Some(TlsVersion::SSL30));
        assert_eq!(TlsVersion::from_u16(769), Some(TlsVersion::TLS10));
        assert_eq!(TlsVersion::from_u16(770), Some(TlsVersion::TLS11));
        assert_eq!(TlsVersion::from_u16(771), Some(TlsVersion::TLS12));
        assert_eq!(TlsVersion::from_u16(772), Some(TlsVersion::TLS13));
        assert_eq!(TlsVersion::from_u16(773), None);
    }

    #[test]
    fn test_parse_header_bytes() {
        let bytes = [0x16, 0x03, 0x03, 0x00, 0x05];
        let (remaining, (content_type, version, length)) = Tls::parse_header_bytes(&bytes).unwrap();
        assert_eq!(content_type, 0x16);
        assert_eq!(version, 0x0303);
        assert_eq!(length, 5);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_tls_from_bytes_valid() {
        let bytes = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x02, 0x03, 0x04, 0x05];
        let tls = Tls::from_bytes(&bytes).unwrap();
        assert_eq!(tls.content_type, TlsContentType::Handshake);
        assert_eq!(tls.version, TlsVersion::TLS12);
        assert_eq!(tls.length, 5);
        assert_eq!(tls.payload, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_tls_from_bytes_invalid() {
        let bytes = [0xFF, 0x03, 0x03, 0x00, 0x05, 0x01, 0x02, 0x03, 0x04, 0x05];
        let tls = Tls::from_bytes(&bytes);
        assert!(tls.is_err());
    }

    #[test]
    fn test_tls_new_valid() {
        let content_type = 0x16; // Handshake
        let version = 0x0303; // TLS12
        let length = 5;
        let payload = [0x01, 0x02, 0x03, 0x04, 0x05];
        let tls = Tls::new(content_type, version, length, &payload).unwrap();
        assert_eq!(tls.content_type, TlsContentType::Handshake);
        assert_eq!(tls.version, TlsVersion::TLS12);
        assert_eq!(tls.length, 5);
        assert_eq!(tls.payload, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_tls_new_invalid_content_type() {
        let content_type = 0xFF;
        let version = 0x0303;
        let length = 5;
        let payload = [0x01, 0x02, 0x03, 0x04, 0x05];
        let tls = Tls::new(content_type, version, length, &payload);
        assert!(tls.is_err());
    }

    #[test]
    fn test_tls_new_invalid_version() {
        let content_type = 0x16;
        let version = 0xFFFF;
        let length = 5;
        let payload = [0x01, 0x02, 0x03, 0x04, 0x05];
        let tls = Tls::new(content_type, version, length, &payload);
        assert!(tls.is_err());
    }
}
