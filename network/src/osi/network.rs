use crate::osi::data_link::{DataLink, DataLinkProtocol};
use log::warn;
use pnet::packet::arp::ArpPacket;
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::icmp::IcmpPacket;
use pnet::packet::icmpv6::Icmpv6Packet;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum NetworkProtocol {
    Arp,
    IcmpIpv4,
    IcmpIpv6,
    Ipv4,
    Ipv6,
    Unknown,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Network {
    pub protocol: NetworkProtocol,
    pub ip: Option<Ip>,
    pub arp: Option<Arp>,
    pub icmp: Option<Icmp>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Arp {
    pub hardware_type: u16,
    pub protocol_type: u16,
    pub operation: u16,
    pub sender_hardware_addr: String,
    pub sender_protocol_addr: String,
    pub target_hardware_addr: String,
    pub target_protocol_addr: String,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Icmp {
    pub code: u8,
    pub _type: u8,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Ip {
    pub source: String,
    pub destination: String,
    pub length: u16,
    pub hop_count: u8,
    pub next_protocol: u8,
}

impl Default for Network {
    fn default() -> Self {
        Network {
            protocol: NetworkProtocol::Unknown,
            ip: None,
            arp: None,
            icmp: None,
            payload: vec![],
        }
    }
}

impl Arp {
    pub fn new(packet: &ArpPacket) -> Arp {
        Arp {
            hardware_type: packet.get_hardware_type().0,
            protocol_type: packet.get_protocol_type().0,
            operation: packet.get_operation().0,
            sender_hardware_addr: packet.get_sender_hw_addr().to_string(),
            sender_protocol_addr: packet.get_sender_proto_addr().to_string(),
            target_hardware_addr: packet.get_target_hw_addr().to_string(),
            target_protocol_addr: packet.get_target_proto_addr().to_string(),
        }
    }
}

impl Icmp {
    pub fn icmpv4(packet: &IcmpPacket) -> Icmp {
        Icmp {
            code: packet.get_icmp_code().0,
            _type: packet.get_icmp_type().0,
        }
    }

    pub fn icmpv6(packet: &Icmpv6Packet) -> Icmp {
        Icmp {
            code: packet.get_icmpv6_code().0,
            _type: packet.get_icmpv6_type().0,
        }
    }
}

impl Ip {
    pub fn ipv4(packet: &Ipv4Packet) -> Ip {
        Ip {
            source: packet.get_source().to_string(),
            destination: packet.get_destination().to_string(),
            length: packet.get_total_length(),
            hop_count: packet.get_ttl(),
            next_protocol: packet.get_next_level_protocol().0,
        }
    }

    pub fn ipv6(packet: &Ipv6Packet) -> Ip {
        Ip {
            source: packet.get_source().to_string(),
            destination: packet.get_destination().to_string(),
            length: packet.get_payload_length(),
            hop_count: packet.get_hop_limit(),
            next_protocol: packet.get_next_header().0,
        }
    }
}

fn parse_arp(packet: &[u8], network: &mut Network) {
    if let Some(arp) = ArpPacket::new(packet) {
        network.arp = Some(Arp::new(&arp));
        network.protocol = NetworkProtocol::Arp;
        network.payload = arp.payload().to_vec();
    } else {
        warn!("Failed to read ARP: {:?}", packet);
    }
}

fn parse_ipv4(packet: &[u8], network: &mut Network) {
    if let Some(ipv4) = Ipv4Packet::new(packet) {
        network.ip = Some(Ip::ipv4(&ipv4));
        network.protocol = NetworkProtocol::Ipv4;
        network.payload = ipv4.payload().to_vec();

        if ipv4.get_next_level_protocol() == IpNextHeaderProtocols::Icmp {
            if let Some(icmp) = IcmpPacket::new(ipv4.payload()) {
                network.icmp = Some(Icmp::icmpv4(&icmp));
                network.protocol = NetworkProtocol::IcmpIpv4;
                network.payload = icmp.payload().to_vec();
            } else {
                warn!("Failed to read IPV4 ICMP: {:?}", ipv4.payload());
            }
        }

    } else {
        warn!("Failed to read IPV4: {:?}", packet);
    }
}

fn parse_ipv6(packet: &[u8], network: &mut Network) {
    if let Some(ipv6) = Ipv6Packet::new(packet) {
        network.ip = Some(Ip::ipv6(&ipv6));
        network.protocol = NetworkProtocol::Ipv6;
        network.payload = ipv6.payload().to_vec();

         if ipv6.get_next_header() == IpNextHeaderProtocols::Icmp {
                 if let Some(icmp) = Icmpv6Packet::new(ipv6.payload()) {
                         network.icmp = Some(Icmp::icmpv6(&icmp));
                         network.protocol = NetworkProtocol::IcmpIpv6;
                         network.payload = icmp.payload().to_vec();
                     } else {
                         warn!("Failed to read IPV6 ICMP: {:?}", ipv6.payload());
                     }
             }

    } else {
        warn!("Failed to read IPV6: {:?}", packet);
    }
}

pub fn read_packet(data_link: &DataLink) -> Network {
    let mut network = Network::default();

    match data_link.protocol {
        DataLinkProtocol::Ethernet => match data_link.ethernet.clone().unwrap().ether_type {
            EtherTypes::Ipv4 => parse_ipv4(&data_link.payload, &mut network),
            EtherTypes::Ipv6 => parse_ipv6(&data_link.payload, &mut network),
            EtherTypes::Arp => parse_arp(&data_link.payload, &mut network),
            _ => {}
        },
        DataLinkProtocol::Unknown => {}
    }

    network
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osi::data_link::Ethernet;
    use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket};
    use pnet::packet::ethernet::{EtherType, EthernetPacket, MutableEthernetPacket};
    use pnet::packet::icmp::{IcmpCode, IcmpTypes, MutableIcmpPacket};
    use pnet::packet::ipv4::MutableIpv4Packet;
    use pnet::packet::ipv6::MutableIpv6Packet;
    use pnet::util::MacAddr;
    use std::net::{Ipv4Addr, Ipv6Addr};

    fn create_mock_ethernet_packet(ether_type: EtherType, payload: &[u8]) -> Vec<u8> {
        let mut ethernet_frame = vec![0u8; 14 + payload.len()];
        {
            let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_frame).unwrap();
            ethernet_packet.set_source(MacAddr::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55));
            ethernet_packet.set_destination(MacAddr::new(0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb));
            ethernet_packet.set_ethertype(ether_type);
            ethernet_packet.set_payload(payload);
        }
        ethernet_frame
    }

    #[test]
    fn test_parse_arp_packet() {
        let mut arp_packet_data = [0u8; 28];
        {
            let mut arp_packet = MutableArpPacket::new(&mut arp_packet_data).unwrap();
            arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
            arp_packet.set_protocol_type(EtherTypes::Ipv4);
            arp_packet.set_operation(ArpOperations::Request);
            arp_packet.set_sender_hw_addr(MacAddr::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55));
            arp_packet.set_sender_proto_addr(Ipv4Addr::new(192, 168, 0, 1));
            arp_packet.set_target_hw_addr(MacAddr::new(0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb));
            arp_packet.set_target_proto_addr(Ipv4Addr::new(192, 168, 0, 2));
        }

        let ethernet_frame = create_mock_ethernet_packet(EtherTypes::Arp, &arp_packet_data);
        let data_link = DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(Ethernet::new(
                &EthernetPacket::new(&ethernet_frame).unwrap(),
            )),
            payload: arp_packet_data.to_vec(),
        };

        let network = read_packet(&data_link);

        assert_eq!(network.protocol, NetworkProtocol::Arp);
        assert!(network.arp.is_some());
        let arp = network.arp.unwrap();
        assert_eq!(arp.hardware_type, ArpHardwareTypes::Ethernet.0);
        assert_eq!(arp.protocol_type, EtherTypes::Ipv4.0);
        assert_eq!(arp.operation, ArpOperations::Request.0);
        assert_eq!(arp.sender_hardware_addr, "00:11:22:33:44:55");
        assert_eq!(arp.sender_protocol_addr, "192.168.0.1");
        assert_eq!(arp.target_hardware_addr, "66:77:88:99:aa:bb");
        assert_eq!(arp.target_protocol_addr, "192.168.0.2");
    }

    #[test]
    fn test_parse_ipv4_packet() {
        let mut ipv4_packet_data = [0u8; 20];
        {
            let mut ipv4_packet = MutableIpv4Packet::new(&mut ipv4_packet_data).unwrap();
            ipv4_packet.set_source(Ipv4Addr::new(192, 168, 0, 1));
            ipv4_packet.set_destination(Ipv4Addr::new(192, 168, 0, 2));
            ipv4_packet.set_total_length(20);
            ipv4_packet.set_ttl(64);
            ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
        }

        let ethernet_frame = create_mock_ethernet_packet(EtherTypes::Ipv4, &ipv4_packet_data);
        let data_link = DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(Ethernet::new(
                &EthernetPacket::new(&ethernet_frame).unwrap(),
            )),
            payload: ipv4_packet_data.to_vec(),
        };

        let network = read_packet(&data_link);

        assert_eq!(network.protocol, NetworkProtocol::Ipv4);
        assert!(network.ip.is_some());
        let ip = network.ip.unwrap();
        assert_eq!(ip.source, "192.168.0.1");
        assert_eq!(ip.destination, "192.168.0.2");
        assert_eq!(ip.length, 20);
        assert_eq!(ip.hop_count, 64);
        assert_eq!(ip.next_protocol, IpNextHeaderProtocols::Icmp.0);
    }

    #[test]
    fn test_parse_icmpv4_packet() {
        let mut icmp_packet_data = [0u8; 8];
        {
            let mut icmp_packet = MutableIcmpPacket::new(&mut icmp_packet_data).unwrap();
            icmp_packet.set_icmp_type(IcmpTypes::EchoReply);
            icmp_packet.set_icmp_code(IcmpCode::new(0));
        }

        let mut ipv4_packet_data = [0u8; 20 + 8];
        {
            let mut ipv4_packet = MutableIpv4Packet::new(&mut ipv4_packet_data).unwrap();
            ipv4_packet.set_source(Ipv4Addr::new(192, 168, 0, 1));
            ipv4_packet.set_destination(Ipv4Addr::new(192, 168, 0, 2));
            ipv4_packet.set_total_length(20 + 8);
            ipv4_packet.set_ttl(64);
            ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
            ipv4_packet.set_payload(&icmp_packet_data);
        }

        let ethernet_frame = create_mock_ethernet_packet(EtherTypes::Ipv4, &ipv4_packet_data);
        let data_link = DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(Ethernet::new(
                &EthernetPacket::new(&ethernet_frame).unwrap(),
            )),
            payload: ipv4_packet_data.to_vec(),
        };

        let network = read_packet(&data_link);

        assert_eq!(network.protocol, NetworkProtocol::IcmpIpv4);
        assert!(network.icmp.is_some());
        let icmp = network.icmp.unwrap();
        assert_eq!(icmp.code, 0);
        assert_eq!(icmp._type, IcmpTypes::EchoReply.0);
    }

    #[test]
    fn test_parse_ipv6_packet() {
        let mut ipv6_packet_data = [0u8; 40];
        {
            let mut ipv6_packet = MutableIpv6Packet::new(&mut ipv6_packet_data).unwrap();
            ipv6_packet.set_source(Ipv6Addr::new(
                0x2001, 0xdb8, 0xac10, 0x0370, 0x7300, 0x12ff, 0xfe00, 0x1001,
            ));
            ipv6_packet.set_destination(Ipv6Addr::new(
                0x2001, 0xdb8, 0xac10, 0x0370, 0x7300, 0x12ff, 0xfe00, 0x1002,
            ));
            ipv6_packet.set_hop_limit(255);
            ipv6_packet.set_payload_length(0);
            ipv6_packet.set_next_header(IpNextHeaderProtocols::Udp);
        }

        let ethernet_frame = create_mock_ethernet_packet(EtherTypes::Ipv6, &ipv6_packet_data);
        let data_link = DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(Ethernet::new(
                &EthernetPacket::new(&ethernet_frame).unwrap(),
            )),
            payload: ipv6_packet_data.to_vec(),
        };

        let network = read_packet(&data_link);

        assert_eq!(network.protocol, NetworkProtocol::Ipv6);
        assert!(network.ip.is_some());
        let ip = network.ip.unwrap();
        assert_eq!(ip.source, "2001:db8:ac10:370:7300:12ff:fe00:1001");
        assert_eq!(ip.destination, "2001:db8:ac10:370:7300:12ff:fe00:1002");
        assert_eq!(ip.hop_count, 255);
        assert_eq!(ip.next_protocol, IpNextHeaderProtocols::Udp.0);
    }

    #[test]
    fn test_parse_unknown_ether_type() {
        let unknown_packet_data = [0xde, 0xad, 0xbe, 0xef]; // Some random data

        let ethernet_frame = create_mock_ethernet_packet(EtherType(0x1234), &unknown_packet_data);
        let data_link = DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(Ethernet::new(
                &EthernetPacket::new(&ethernet_frame).unwrap(),
            )),
            payload: unknown_packet_data.to_vec(),
        };

        let network = read_packet(&data_link);

        assert_eq!(network.protocol, NetworkProtocol::Unknown);
        assert!(network.ip.is_none());
        assert!(network.arp.is_none());
        assert!(network.icmp.is_none());
        assert_eq!(network.payload, []);
    }
}
