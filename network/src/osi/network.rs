use pnet::packet::arp::ArpPacket;
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::icmp::IcmpPacket;
use pnet::packet::icmpv6::Icmpv6Packet;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use crate::osi::data_link::{DataLink, DataLinkProtocol};

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

impl Network {
    fn new() -> Network {
        Network {
            protocol: NetworkProtocol::Unknown,
            ip: None,
            arp: None,
            icmp: None,
            payload: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Ip {
    pub source: String,
    pub destination: String,
    pub length: u16,
    pub hop_count: u8,
    pub next_protocol: u8,
}

impl Ip {
    fn ipv4(packet: &Ipv4Packet) -> Ip {
        Ip {
            source: packet.get_source().to_string(),
            destination: packet.get_destination().to_string(),
            length: packet.get_total_length(),
            hop_count: packet.get_ttl(),
            next_protocol: packet.get_next_level_protocol().0,
        }
    }

    fn ipv6(packet: &Ipv6Packet) -> Ip {
        Ip {
            source: packet.get_source().to_string(),
            destination: packet.get_destination().to_string(),
            length: packet.get_payload_length(),
            hop_count: packet.get_hop_limit(),
            next_protocol: packet.get_next_header().0,
        }
    }
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

impl Arp {
    fn new(packet: &ArpPacket) -> Arp {
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

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Icmp {
    code: u8,
    _type: u8,
}

impl Icmp {

    fn icmpv4(icmpv4: &IcmpPacket) -> Icmp {
        Icmp {
            code: icmpv4.get_icmp_code().0,
            _type: icmpv4.get_icmp_type().0,
        }
    }

    fn icmpv6(icmpv6: &Icmpv6Packet) -> Icmp {
        Icmp {
            code: icmpv6.get_icmpv6_code().0,
            _type: icmpv6.get_icmpv6_type().0,
        }
    }


}


fn parse_ipv4(packet: &[u8], network: &mut Network) -> () {

    if let Some(ipv4) = Ipv4Packet::new(packet) {
        network.ip = Some(Ip::ipv4(&ipv4));
        network.protocol = NetworkProtocol::Ipv4;
        network.payload = ipv4.payload().to_vec();

        match ipv4.get_next_level_protocol() {
            IpNextHeaderProtocols::Icmp => {
                let icmp = IcmpPacket::new(ipv4.payload()).unwrap();
                network.icmp = Some(Icmp::icmpv4(&icmp));
                network.protocol = NetworkProtocol::IcmpIpv4;
                network.payload = icmp.payload().to_vec();
            }
            _ => {
                // TODO
            }

        }
    }

}

fn parse_ipv6(packet: &[u8], network: &mut Network)  {
    if let Some(ipv6) = Ipv6Packet::new(&packet) {
        network.ip = Some(Ip::ipv6(&ipv6));
        network.protocol = NetworkProtocol::Ipv6;
        network.payload = ipv6.payload().to_vec();

        match ipv6.get_next_header() {
            IpNextHeaderProtocols::Icmp => {
                let icmp = Icmpv6Packet::new(ipv6.payload()).unwrap();
                network.icmp = Some(Icmp::icmpv6(&icmp));
                network.protocol = NetworkProtocol::IcmpIpv6;
                network.payload = icmp.payload().to_vec();
            }

            _ => {
                // TODO
            }
        }

    }
}

fn parse_arp(packet: &[u8], network: &mut Network)  {
    if let Some(arp) = ArpPacket::new(&packet) {
        network.arp = Some(Arp::new(&arp));
        network.protocol = NetworkProtocol::Arp;
        network.payload = arp.payload().to_vec();
    }
}

pub fn read_data_link_packet(data_link: &DataLink) -> Network {
    let mut network = Network::new();
    let data_link = data_link.clone();

    match data_link.protocol {
        DataLinkProtocol::Ethernet => {

            match data_link.ethernet.unwrap().ether_type {
                EtherTypes::Ipv4 => parse_ipv4(&data_link.payload, &mut network),
                EtherTypes::Ipv6 => parse_ipv6(&data_link.payload, &mut network),
                EtherTypes::Arp => parse_arp(&data_link.payload, &mut network),
                _ => {
                    // todo
                }
            }

        },
        DataLinkProtocol::Unknown => {
            // ToDO}
        }
    }

    network
}
