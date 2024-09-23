use crate::error::Error;
use nom::bytes::complete::take;
use nom::number::complete::be_u16;
use nom::IResult;
use std::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum TlsContentType {
    ChangeCipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    ApplicationData = 23,
    Heartbeat = 24,
}

#[repr(u16)]
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

impl TryFrom<u8> for TlsContentType {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0x14 => Ok(TlsContentType::ChangeCipherSpec),
            0x15 => Ok(TlsContentType::Alert),
            0x16 => Ok(TlsContentType::Handshake),
            0x17 => Ok(TlsContentType::ApplicationData),
            0x18 => Ok(TlsContentType::Heartbeat),
            _ => Err(()),
        }
    }
}

impl From<TlsContentType> for u8 {
    fn from(content_type: TlsContentType) -> Self {
        content_type as u8
    }
}

impl TryFrom<u16> for TlsVersion {
    type Error = ();

    fn try_from(version: u16) -> Result<Self, Self::Error> {
        match version {
            2 => Ok(TlsVersion::SSL20),
            768 => Ok(TlsVersion::SSL30),
            769 => Ok(TlsVersion::TLS10),
            770 => Ok(TlsVersion::TLS11),
            771 => Ok(TlsVersion::TLS12),
            772 => Ok(TlsVersion::TLS13),
            _ => Err(()),
        }
    }
}

impl From<TlsVersion> for u16 {
    fn from(version: TlsVersion) -> Self {
        version as u16
    }
}

impl Tls {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if let Ok((bytes, (content_type, version, length))) = Self::parse_header_bytes(bytes) {
            Ok(Self::new(content_type, version, length, bytes)?)
        } else {
            Err(Error::PacketParsing)
        }
    }

    fn new(content_type: u8, version: u16, length: u16, payload: &[u8]) -> Result<Self, Error> {
        let content_type: TlsContentType =
            TlsContentType::try_from(content_type).map_err(|_| Error::PacketParsing)?;
        let version: TlsVersion =
            TlsVersion::try_from(version).map_err(|_| Error::PacketParsing)?;
        Ok(Tls {
            content_type,
            version,
            length,
            payload: payload.to_vec(),
        })
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
    fn test_tls_content_type_try_from() {
        assert_eq!(
            TlsContentType::try_from(0x14).unwrap(),
            TlsContentType::ChangeCipherSpec
        );
        assert_eq!(
            TlsContentType::try_from(0x15).unwrap(),
            TlsContentType::Alert
        );
        assert_eq!(
            TlsContentType::try_from(0x16).unwrap(),
            TlsContentType::Handshake
        );
        assert_eq!(
            TlsContentType::try_from(0x17).unwrap(),
            TlsContentType::ApplicationData
        );
        assert_eq!(
            TlsContentType::try_from(0x18).unwrap(),
            TlsContentType::Heartbeat
        );
        assert!(TlsContentType::try_from(0x19).is_err());
    }

    #[test]
    fn test_tls_version_try_from() {
        assert_eq!(TlsVersion::try_from(2).unwrap(), TlsVersion::SSL20);
        assert_eq!(TlsVersion::try_from(768).unwrap(), TlsVersion::SSL30);
        assert_eq!(TlsVersion::try_from(769).unwrap(), TlsVersion::TLS10);
        assert_eq!(TlsVersion::try_from(770).unwrap(), TlsVersion::TLS11);
        assert_eq!(TlsVersion::try_from(771).unwrap(), TlsVersion::TLS12);
        assert_eq!(TlsVersion::try_from(772).unwrap(), TlsVersion::TLS13);
        assert!(TlsVersion::try_from(773).is_err());
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
