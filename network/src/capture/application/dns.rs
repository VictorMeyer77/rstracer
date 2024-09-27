use pnet::packet::dns::{Dns, DnsPacket};
use pnet::packet::{FromPacket, PrimitiveValues};
use std::panic;
use std::panic::{set_hook, take_hook, AssertUnwindSafe};

pub fn from_bytes(bytes: &[u8]) -> Option<Dns> {
    // todo check 'pnet' crate future evolution to handle DNS as other components
    let original_hook = take_hook();
    set_hook(Box::new(|_| {}));
    let dns = panic::catch_unwind(AssertUnwindSafe(|| {
        DnsPacket::new(bytes).unwrap().from_packet()
    }));
    set_hook(original_hook);

    if dns.is_ok() {
        let dns = dns.unwrap();
        if valid_header(&dns) {
            Some(dns)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn valid_header(dns: &Dns) -> bool {
    (dns.query_count > 0 || dns.response_count > 0)
        && (dns.query_count as usize == dns.queries.len()
            && dns.response_count as usize == dns.responses.len())
        && (dns.opcode.to_primitive_values().0 < 16 && dns.rcode.to_primitive_values().0 < 16)
}

#[cfg(test)]
pub mod tests {
    use crate::capture::application::dns::{from_bytes, valid_header};

    pub fn create_dns_packet() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&create_dns_header());
        bytes.extend_from_slice(&create_dns_query());
        bytes.extend_from_slice(&create_dns_record());
        bytes.extend_from_slice(&create_dns_record());
        bytes
    }

    fn create_dns_header() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0x12, 0x34]);
        bytes.extend_from_slice(&[0x81, 0x80]);
        bytes.extend_from_slice(&[0x00, 0x01]);
        bytes.extend_from_slice(&[0x00, 0x01]);
        bytes.extend_from_slice(&[0x00, 0x00]);
        bytes.extend_from_slice(&[0x00, 0x00]);
        bytes
    }

    fn create_dns_query() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[
            3, 119, 119, 119, 7, 101, 120, 97, 109, 112, 108, 101, 3, 99, 111, 109, 0,
        ]);
        bytes.extend_from_slice(&[0x00, 0x01]);
        bytes.extend_from_slice(&[0x00, 0x01]);
        bytes
    }

    fn create_dns_record() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[
            0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3c, 0x00, 0x04, 0x5d, 0xb8,
            0xd8, 0x22,
        ]);
        bytes
    }

    #[test]
    fn test_from_bytes_valid_packet() {
        let bytes = create_dns_packet();
        let dns = from_bytes(&bytes).unwrap();

        assert!(valid_header(&dns));
        assert_eq!(
            &String::from_utf8_lossy(&dns.queries.first().unwrap().qname),
            "\u{3}www\u{7}example\u{3}com\0"
        );
        assert_eq!(&dns.responses.first().unwrap().data, &[93, 184, 216, 34]);
    }

    #[test]
    fn test_from_bytes_invalid_packet() {
        let bytes = vec![0; 10];
        let dns = from_bytes(&bytes);
        assert!(dns.is_none());
    }

    #[test]
    fn test_from_bytes_invalid_header() {
        let bytes = create_dns_packet();
        let mut dns = from_bytes(&bytes).unwrap();
        dns.query_count = 4141;
        assert!(!valid_header(&dns));
    }
}
