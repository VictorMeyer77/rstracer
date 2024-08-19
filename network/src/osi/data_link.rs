use pnet::packet::ethernet::{EtherType, EthernetPacket};
use pnet::packet::Packet;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum DataLinkProtocol {
    Ethernet,
    Unknown,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct DataLink {
    pub protocol: DataLinkProtocol,
    pub ethernet: Option<Ethernet>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Ethernet {
    pub source: String,      // MacAddr
    pub destination: String, // MacAddr
    pub ether_type: EtherType,
}

impl Ethernet {
    pub fn new(packet: &EthernetPacket) -> Ethernet {
        Ethernet {
            source: packet.get_source().to_string(),
            destination: packet.get_destination().to_string(),
            ether_type: packet.get_ethertype(),
        }
    }
}

pub fn read_packet(packet: &[u8]) -> DataLink {
    if let Some(ethernet) = EthernetPacket::new(packet) {
        DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(Ethernet::new(&ethernet)),
            payload: ethernet.payload().to_vec(),
        }
    } else {
        DataLink {
            protocol: DataLinkProtocol::Unknown,
            ethernet: None,
            payload: packet.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket};
    use pnet::util::MacAddr;

    #[test]
    fn test_ethernet_new() {
        let mut packet_data = [0u8; 14];
        let mut ethernet_packet = MutableEthernetPacket::new(&mut packet_data).unwrap();
        ethernet_packet.set_source(MacAddr::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55));
        ethernet_packet.set_destination(MacAddr::new(0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb));
        ethernet_packet.set_ethertype(EtherTypes::Ipv4);

        let ethernet = Ethernet::new(&EthernetPacket::new(ethernet_packet.packet()).unwrap());

        assert_eq!(ethernet.source, "00:11:22:33:44:55");
        assert_eq!(ethernet.destination, "66:77:88:99:aa:bb");
        assert_eq!(ethernet.ether_type, EtherTypes::Ipv4);
    }

    #[test]
    fn test_read_packet_with_ethernet() {
        let mut packet_data = [0u8; 18];
        let mut ethernet_packet = MutableEthernetPacket::new(&mut packet_data).unwrap();
        ethernet_packet.set_source(MacAddr::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55));
        ethernet_packet.set_destination(MacAddr::new(0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb));
        ethernet_packet.set_ethertype(EtherTypes::Ipv4);
        ethernet_packet.set_payload(&[0xde, 0xad, 0xbe, 0xef]);

        let data_link = read_packet(ethernet_packet.packet());
        assert_eq!(data_link.protocol, DataLinkProtocol::Ethernet);
        assert!(data_link.ethernet.is_some());

        let ethernet = data_link.ethernet.unwrap();
        assert_eq!(ethernet.source, "00:11:22:33:44:55");
        assert_eq!(ethernet.destination, "66:77:88:99:aa:bb");
        assert_eq!(ethernet.ether_type, EtherTypes::Ipv4);
        assert_eq!(data_link.payload, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn test_read_packet_with_unknown_protocol() {
        let packet_data = [0xde, 0xad, 0xbe, 0xef];

        let data_link = read_packet(&packet_data);

        assert_eq!(data_link.protocol, DataLinkProtocol::Unknown);
        assert!(data_link.ethernet.is_none());
        assert_eq!(data_link.payload, packet_data.to_vec());
    }
}
