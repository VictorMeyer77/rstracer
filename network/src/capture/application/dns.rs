use crate::capture::application::ApplicationProtocol;
use crate::capture::Layer;
use crate::error::Error;
use nom::bytes::complete::take;
use nom::number::complete::{be_u16, be_u32};
use nom::IResult;

#[derive(Debug, Clone)]
pub struct Dns {
    pub header: DnsHeader,
    pub question: DnsQuestion,
    pub answer: DnsRecord,
    pub additional: Vec<DnsRecord>,
}

#[derive(Debug, Clone)]
struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub qd_count: u16,
    pub an_count: u16,
    pub ns_count: u16,
    pub ar_count: u16,
}

#[derive(Debug, Clone)]
struct DnsHeaderFlags {
    pub qr: bool,
    pub opcode: u8,
    pub aa: bool,
    pub tc: bool,
    pub rd: bool,
    pub ra: bool,
    pub z: u8,
    pub rcode: u8,
}

#[derive(Debug, Clone)]
pub struct DnsQuestion {
    pub qname: String,
    pub qtype: u16,
    pub qclass: u16,
}

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub name: String,
    pub rr_type: u16,
    pub rr_class: u16,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

impl Dns {
    pub fn from_bytes(bytes: &[u8]) -> Result<Dns, Error> {
        if let Ok((bytes, header)) = DnsHeader::from_bytes(bytes) {
            if header.valid_header() {
                if let Ok((bytes, question)) = DnsQuestion::from_bytes(bytes) {
                    if let Ok((bytes, answer)) = DnsRecord::from_bytes(bytes) {
                        if let Ok((_, additional)) = Self::parse_additional(bytes) {
                            return Ok(Dns {
                                header,
                                question,
                                answer,
                                additional,
                            });
                        }
                    }
                }
            }
        }
        Err(Error::PacketParseError {
            layer: Layer::Application.to_string(),
            protocol: ApplicationProtocol::Dns.to_string(),
            data: vec![],
        })
    }

    fn parse_additional(bytes: &[u8]) -> IResult<&[u8], Vec<DnsRecord>> {
        let mut additional = Vec::new();
        let mut bytes = bytes;

        while !bytes.is_empty() {
            let (rest, record) = DnsRecord::from_bytes(bytes)?;
            bytes = rest;
            additional.push(record);
        }

        Ok((bytes, additional))
    }
}

impl DnsHeader {
    fn from_bytes(bytes: &[u8]) -> IResult<&[u8], DnsHeader> {
        let (bytes, id) = be_u16(bytes)?;
        let (bytes, flags) = be_u16(bytes)?;
        let (bytes, qd_count) = be_u16(bytes)?;
        let (bytes, an_count) = be_u16(bytes)?;
        let (bytes, ns_count) = be_u16(bytes)?;
        let (bytes, ar_count) = be_u16(bytes)?;

        Ok((
            bytes,
            DnsHeader {
                id,
                flags,
                qd_count,
                an_count,
                ns_count,
                ar_count,
            },
        ))
    }

    pub fn get_flags(&self) -> DnsHeaderFlags {
        DnsHeaderFlags::from_u16(self.flags)
    }

    pub fn valid_header(&self) -> bool {
        let flags = self.get_flags();
        (self.qd_count > 0 || self.ar_count > 0)
            && (self.qd_count < 2)
            && (flags.opcode < 16 && flags.rcode < 16 && flags.z == 0)
    }
}

impl DnsHeaderFlags {
    fn from_u16(flags: u16) -> Self {
        DnsHeaderFlags {
            qr: (flags & 0x8000) != 0,
            opcode: ((flags & 0x7800) >> 11) as u8,
            aa: (flags & 0x0400) != 0,
            tc: (flags & 0x0200) != 0,
            rd: (flags & 0x0100) != 0,
            ra: (flags & 0x0080) != 0,
            z: ((flags & 0x0070) >> 4) as u8,
            rcode: (flags & 0x000F) as u8,
        }
    }
}

impl DnsQuestion {
    fn from_bytes(bytes: &[u8]) -> IResult<&[u8], DnsQuestion> {
        let (bytes, qname) = Self::parse_qname(bytes)?;
        let (bytes, qtype) = be_u16(bytes)?;
        let (bytes, qclass) = be_u16(bytes)?;

        Ok((
            bytes,
            DnsQuestion {
                qname,
                qtype,
                qclass,
            },
        ))
    }

    fn parse_qname(bytes: &[u8]) -> IResult<&[u8], String> {
        let mut bytes = bytes;
        let mut labels = Vec::new();

        loop {
            let (rest, header) = take(1usize)(bytes)?;

            let len = header[0] as usize;
            if len == 0 {
                bytes = rest;
                break;
            } else {
                let (rest, label) = take(len)(rest)?;
                labels.push(String::from_utf8_lossy(label).into_owned());
                bytes = rest;
            }
        }

        Ok((bytes, labels.join(".")))
    }
}

impl DnsRecord {
    fn from_bytes(bytes: &[u8]) -> IResult<&[u8], DnsRecord> {
        let (bytes, name) = Self::parse_name(bytes)?;
        let (bytes, rr_type) = be_u16(bytes)?;
        let (bytes, rr_class) = be_u16(bytes)?;
        let (bytes, ttl) = be_u32(bytes)?;
        let (bytes, rdlength) = be_u16(bytes)?;
        let (bytes, rdata) = take(rdlength)(bytes)?;
        Ok((
            bytes,
            DnsRecord {
                name,
                rr_type,
                rr_class,
                ttl,
                rdlength,
                rdata: rdata.to_vec(),
            },
        ))
    }

    fn parse_name(bytes: &[u8]) -> IResult<&[u8], String> {
        let (remaining, header) = take(1usize)(bytes)?;
        if header[0] & 0xC0 == 0xC0 {
            // todo
            let (remaining, _) = take(1usize)(remaining)?;
            // let offset = ((header[0] as usize & 0x3F) << 8) | pointer[0] as usize;
            Ok((remaining, "0xC0".to_string()))
        } else {
            DnsQuestion::parse_qname(bytes)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn create_dns_packet() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&create_dns_header());
        bytes.extend_from_slice(&create_dns_question());
        bytes.extend_from_slice(&create_dns_record());
        bytes.extend_from_slice(&create_dns_record());
        bytes
    }

    fn create_dns_header() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0x12, 0x34]); // ID = 0x1234
        bytes.extend_from_slice(&[0x81, 0x80]); // Flags = 0x8180 (response, recursion available, no error)
        bytes.extend_from_slice(&[0x00, 0x01]); // QDCOUNT = 1
        bytes.extend_from_slice(&[0x00, 0x01]); // ANCOUNT = 1
        bytes.extend_from_slice(&[0x00, 0x00]); // NSCOUNT = 0
        bytes.extend_from_slice(&[0x00, 0x00]); // ARCOUNT = 0
        bytes
    }

    fn create_dns_question() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[
            3, 119, 119, 119, // "www"
            7, 101, 120, 97, 109, 112, 108, 101, // "example"
            3, 99, 111, 109, // "com"
            0,   // End of QNAME
        ]);
        bytes.extend_from_slice(&[0x00, 0x01]); // QTYPE = A
        bytes.extend_from_slice(&[0x00, 0x01]); // QCLASS = IN
        bytes
    }

    fn create_dns_record() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[
            0xc0, 0x0c, // NAME = Pointer to offset 0x0c (the start of QNAME)
            0x00, 0x01, // TYPE = A
            0x00, 0x01, // CLASS = IN
            0x00, 0x00, 0x00, 0x3c, // TTL = 60 seconds
            0x00, 0x04, // RDLENGTH = 4 bytes
            0x5d, 0xb8, 0xd8, 0x22, // RDATA = 93.184.216.34 (IPv4 address)
        ]);
        bytes
    }

    #[test]
    fn test_dns_from_bytes() {
        let bytes = create_dns_packet();
        let dns = Dns::from_bytes(&bytes).unwrap();

        assert_eq!(dns.header.id, 0x1234);
        assert_eq!(dns.header.flags, 0x8180);
        assert_eq!(dns.question.qname, "www.example.com");
        assert_eq!(dns.answer.name, "0xC0");
        assert_eq!(dns.additional.len(), 1);
    }

    #[test]
    fn test_dns_parse_additional() {
        let mut bytes = vec![];
        bytes.extend_from_slice(create_dns_record().as_slice());
        bytes.extend_from_slice(create_dns_record().as_slice());
        bytes.extend_from_slice(create_dns_record().as_slice());

        let (remaining, records) = Dns::parse_additional(&bytes).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn test_dns_header_from_bytes() {
        let bytes = create_dns_header();
        let (remaining, header) = DnsHeader::from_bytes(&bytes).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(header.id, 0x1234);
        assert_eq!(header.flags, 0x8180);
        assert_eq!(header.qd_count, 0x0001);
        assert_eq!(header.an_count, 0x0001);
        assert_eq!(header.ns_count, 0x0000);
        assert_eq!(header.ar_count, 0x0000);
    }

    #[test]
    fn test_dns_flags_from_u16() {
        let flags = 0x8180;
        let dns_flags = DnsHeaderFlags::from_u16(flags);

        assert_eq!(dns_flags.qr, true);
        assert_eq!(dns_flags.opcode, 0);
        assert_eq!(dns_flags.aa, false);
        assert_eq!(dns_flags.tc, false);
        assert_eq!(dns_flags.rd, true);
        assert_eq!(dns_flags.ra, true);
        assert_eq!(dns_flags.z, 0);
        assert_eq!(dns_flags.rcode, 0);
    }

    #[test]
    fn test_dns_flags_edge_cases() {
        let flags = 0xFFFF;
        let dns_flags = DnsHeaderFlags::from_u16(flags);

        assert_eq!(dns_flags.qr, true);
        assert_eq!(dns_flags.opcode, 15);
        assert_eq!(dns_flags.aa, true);
        assert_eq!(dns_flags.tc, true);
        assert_eq!(dns_flags.rd, true);
        assert_eq!(dns_flags.ra, true);
        assert_eq!(dns_flags.z, 7);
        assert_eq!(dns_flags.rcode, 15);
    }

    #[test]
    fn test_dns_flags_zero() {
        let flags = 0x0000;
        let dns_flags = DnsHeaderFlags::from_u16(flags);

        assert_eq!(dns_flags.qr, false);
        assert_eq!(dns_flags.opcode, 0);
        assert_eq!(dns_flags.aa, false);
        assert_eq!(dns_flags.tc, false);
        assert_eq!(dns_flags.rd, false);
        assert_eq!(dns_flags.ra, false);
        assert_eq!(dns_flags.z, 0);
        assert_eq!(dns_flags.rcode, 0);
    }

    #[test]
    fn test_valid_header() {
        let header = DnsHeader {
            id: 0x1234,
            flags: 0x0100,
            qd_count: 1,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        };
        assert!(header.valid_header());
    }

    #[test]
    fn test_invalid_header_no_questions_or_additional() {
        let header = DnsHeader {
            id: 0x1234,
            flags: 0x0100,
            qd_count: 0,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        };
        assert!(!header.valid_header());
    }

    #[test]
    fn test_dns_question_from_bytes() {
        let bytes = create_dns_question();
        let (remaining, question) = DnsQuestion::from_bytes(&bytes).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(question.qname, "www.example.com");
        assert_eq!(question.qtype, 0x0001);
        assert_eq!(question.qclass, 0x0001);
    }

    #[test]
    fn test_dns_question_parse_qname() {
        let bytes = create_dns_question();
        let (remaining, question) = DnsQuestion::parse_qname(&bytes).unwrap();

        assert_eq!(remaining.len(), 4);
        assert_eq!(question, "www.example.com");
    }

    #[test]
    fn test_dns_record_parse_name_with_full_name() {
        let bytes: [u8; 31] = [
            3, 119, 119, 119, // "www"
            7, 101, 120, 97, 109, 112, 108, 101, // "example"
            3, 99, 111, 109, // "com"
            0,   // end of qname
            0x00, 0x01, // rr_type A
            0x00, 0x01, // rr_class IN
            0x00, 0x00, 0x01, 0x2c, // TTL 300
            0x00, 0x04, // RDLENGTH 4
            0xc0, 0x00, 0x02, 0x01, // RDATA 192.0.2.1
        ];

        let (remaining, record) = DnsRecord::parse_name(&bytes).unwrap();
        assert_eq!(remaining.len(), 14);
        assert_eq!(record, "www.example.com");
    }

    #[test]
    fn test_dns_record_parse_name_with_pointer() {
        let bytes: [u8; 16] = [
            0xC0, 0x0C, 0x00, 0x01, // rr_type A
            0x00, 0x01, // rr_class IN
            0x00, 0x00, 0x01, 0x2c, // TTL 300
            0x00, 0x04, // RDLENGTH 4
            0xc0, 0x00, 0x02, 0x01, // RDATA 192.0.2.1
        ];

        let (remaining, record) = DnsRecord::parse_name(&bytes).unwrap();
        assert_eq!(remaining.len(), 14);
        assert_eq!(record, "0xC0");
    }

    #[test]
    fn test_dns_record_from_bytes() {
        let record = create_dns_record();
        let (remaining, parsed_record) = DnsRecord::from_bytes(&record).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(parsed_record.name, "0xC0");
        assert_eq!(parsed_record.rr_type, 0x0001);
        assert_eq!(parsed_record.rr_class, 0x0001);
        assert_eq!(parsed_record.ttl, 60);
        assert_eq!(parsed_record.rdlength, 4);
        assert_eq!(parsed_record.rdata, vec![0x5d, 0xb8, 0xd8, 0x22]);
    }
}
