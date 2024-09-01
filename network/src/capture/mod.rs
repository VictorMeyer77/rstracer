use crate::capture::application::Application;
use crate::capture::data_link::DataLink;
use crate::capture::network::Network;
use crate::capture::transport::Transport;
use crate::error::Error;
use chrono::Local;
use pcap::Device;
use std::fmt;

pub mod application;
pub mod data_link;
pub mod network;
pub mod transport;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Layer {
    DataLink,
    Network,
    Transport,
    Application,
}

#[derive(Debug, Clone)]
pub struct Capture {
    pub device: Device,
    pub packet: Vec<u8>,
    pub created_at: i64,
    pub data_link: Option<DataLink>,
    pub network: Option<Network>,
    pub transport: Option<Transport>,
    pub application: Option<Application>,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Layer::DataLink => "data_link",
                Layer::Network => "network",
                Layer::Transport => "transport",
                Layer::Application => "application",
            }
        )
    }
}

impl Capture {
    fn new(packet: &[u8], device: &Device) -> Capture {
        Capture {
            device: device.clone(),
            packet: packet.to_vec(),
            created_at: Local::now().timestamp_millis(),
            data_link: None,
            network: None,
            transport: None,
            application: None,
        }
    }

    pub fn parse(packet: &[u8], device: &Device) -> Result<Capture, Error> {
        let mut capture = Self::new(packet, device);
        match data_link::read_packet(packet) {
            Ok(data_link) => {
                match network::read_packet(&data_link) {
                    Ok(network) => {
                        match transport::read_packet(&network) {
                            Ok(transport) => {
                                match application::read_packet(&transport) {
                                    Ok(application) => capture.application = Some(application),
                                    Err(error) => return Err(error),
                                }
                                capture.transport = Some(transport)
                            }
                            Err(error) => return Err(error),
                        }
                        capture.network = Some(network)
                    }
                    Err(error) => return Err(error),
                }
                capture.data_link = Some(data_link);
            }
            Err(error) => return Err(error),
        }

        Ok(capture)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::capture::application::ApplicationProtocol;
    use crate::capture::data_link::DataLinkProtocol;
    use crate::capture::network::NetworkProtocol;
    use crate::capture::transport::TransportProtocol;
    use pnet::datalink::MacAddr;
    use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket};
    use pnet::packet::ethernet::{EtherType, EtherTypes, MutableEthernetPacket};
    use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
    use pnet::packet::icmp::IcmpTypes;
    use pnet::packet::icmpv6::{Icmpv6Types, MutableIcmpv6Packet};
    use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
    use pnet::packet::ipv4;
    use pnet::packet::ipv4::MutableIpv4Packet;
    use pnet::packet::ipv6::MutableIpv6Packet;
    use pnet::packet::tcp::MutableTcpPacket;
    use pnet::packet::udp::MutableUdpPacket;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    pub fn create_packet() -> Vec<u8> {
        vec![
            204, 45, 27, 186, 195, 248, 248, 99, 63, 244, 10, 21, 8, 0, 69, 0, 0, 67, 129, 205, 0,
            0, 64, 17, 117, 60, 192, 168, 1, 79, 192, 168, 1, 1, 174, 55, 0, 53, 0, 47, 113, 146,
            86, 48, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 6, 116, 97, 105, 118, 101, 109, 3, 99, 111, 109,
            0, 0, 1, 0, 1, 0, 0, 41, 5, 192, 0, 0, 0, 0, 0, 0,
        ]
    }

    // helper functions for mod testing

    pub fn create_ethernet_packet(ether_type: EtherType, payload: &[u8]) -> Vec<u8> {
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

    pub fn create_arp_packet() -> Vec<u8> {
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
        create_ethernet_packet(EtherTypes::Arp, &arp_packet_data)
    }

    pub fn create_ipv4_packet(next_protocol: IpNextHeaderProtocol, payload: &[u8]) -> Vec<u8> {
        let mut ipv4_packet_data = vec![0u8; 20 + payload.len()];
        {
            let mut ipv4_packet = MutableIpv4Packet::new(&mut ipv4_packet_data).unwrap();
            ipv4_packet.set_version(4);
            ipv4_packet.set_header_length(5);
            ipv4_packet.set_total_length(20 + payload.len() as u16);
            ipv4_packet.set_ttl(64);
            ipv4_packet.set_next_level_protocol(next_protocol);
            ipv4_packet.set_source(Ipv4Addr::new(192, 168, 0, 1));
            ipv4_packet.set_destination(Ipv4Addr::new(192, 168, 0, 2));
            ipv4_packet.set_payload(payload);
            let checksum = ipv4::checksum(&ipv4_packet.to_immutable());
            ipv4_packet.set_checksum(checksum);
        }
        create_ethernet_packet(EtherTypes::Ipv4, &ipv4_packet_data)
    }

    pub fn create_ipv6_packet(next_protocol: IpNextHeaderProtocol, payload: &[u8]) -> Vec<u8> {
        let mut ipv6_packet_data = vec![0u8; 40 + payload.len()];
        {
            let mut ipv6_packet = MutableIpv6Packet::new(&mut ipv6_packet_data).unwrap();
            ipv6_packet.set_version(6);
            ipv6_packet.set_payload_length(payload.len() as u16);
            ipv6_packet.set_next_header(next_protocol);
            ipv6_packet.set_hop_limit(64);
            ipv6_packet.set_source(Ipv6Addr::from_str("2001:db8::1").unwrap());
            ipv6_packet.set_destination(Ipv6Addr::from_str("2001:db8::2").unwrap());
            ipv6_packet.set_payload(payload);
        }
        create_ethernet_packet(EtherTypes::Ipv6, &ipv6_packet_data)
    }

    pub fn create_tcp_packet(src_port: u16, dst_port: u16, payload: &[u8]) -> Vec<u8> {
        let mut tcp_packet_data = vec![0u8; 20 + payload.len()];
        {
            let mut tcp_packet = MutableTcpPacket::new(&mut tcp_packet_data).unwrap();
            tcp_packet.set_source(src_port);
            tcp_packet.set_destination(dst_port);
            tcp_packet.set_sequence(1);
            tcp_packet.set_acknowledgement(1);
            tcp_packet.set_data_offset(5);
            tcp_packet.set_flags(0b000101000); // SYN and ACK flags
            tcp_packet.set_window(65535);
            tcp_packet.set_checksum(0);
            tcp_packet.set_urgent_ptr(0);
            tcp_packet.set_payload(payload);
            let checksum = pnet::packet::tcp::ipv4_checksum(
                &tcp_packet.to_immutable(),
                &Ipv4Addr::new(192, 168, 0, 1),
                &Ipv4Addr::new(192, 168, 0, 2),
            );
            tcp_packet.set_checksum(checksum);
        }
        create_ipv4_packet(IpNextHeaderProtocols::Tcp, &tcp_packet_data)
    }

    pub fn create_udp_packet(src_port: u16, dst_port: u16, payload: &[u8]) -> Vec<u8> {
        let mut udp_packet_data = vec![0u8; 8 + payload.len()];
        {
            let mut udp_packet = MutableUdpPacket::new(&mut udp_packet_data).unwrap();
            udp_packet.set_source(src_port);
            udp_packet.set_destination(dst_port);
            udp_packet.set_length((8 + payload.len()) as u16);
            udp_packet.set_checksum(0);
            udp_packet.set_payload(payload);
            let checksum = pnet::packet::udp::ipv4_checksum(
                &udp_packet.to_immutable(),
                &Ipv4Addr::new(192, 168, 0, 1),
                &Ipv4Addr::new(192, 168, 0, 2),
            );
            udp_packet.set_checksum(checksum);
        }
        create_ipv4_packet(IpNextHeaderProtocols::Udp, &udp_packet_data)
    }

    pub fn create_icmpv4_packet(payload: &[u8]) -> Vec<u8> {
        let mut icmp_packet_data = vec![0u8; 8 + payload.len()];
        {
            let mut icmp_packet = MutableEchoRequestPacket::new(&mut icmp_packet_data).unwrap();
            icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
            icmp_packet.set_identifier(0);
            icmp_packet.set_sequence_number(0);
            icmp_packet.set_payload(payload);
        }
        create_ipv4_packet(IpNextHeaderProtocols::Icmp, &icmp_packet_data)
    }

    pub fn create_icmpv6_packet(payload: &[u8]) -> Vec<u8> {
        let mut icmpv6_packet_data = vec![0u8; 8 + payload.len()];
        {
            let mut icmpv6_packet = MutableIcmpv6Packet::new(&mut icmpv6_packet_data).unwrap();
            icmpv6_packet.set_icmpv6_type(Icmpv6Types::EchoRequest);
            icmpv6_packet.set_payload(payload);
        }
        create_ipv6_packet(IpNextHeaderProtocols::Icmpv6, &icmpv6_packet_data)
    }

    #[test]
    fn test_layer_display() {
        assert_eq!(Layer::DataLink.to_string(), "data_link");
        assert_eq!(Layer::Network.to_string(), "network");
        assert_eq!(Layer::Transport.to_string(), "transport");
        assert_eq!(Layer::Application.to_string(), "application");
    }

    #[test]
    fn test_capture_parse() {
        let device = Device::lookup().unwrap().unwrap();
        let packet = create_packet();
        let capture = Capture::parse(&packet, &device).unwrap();

        let data_link = capture.data_link.unwrap();
        assert_eq!(data_link.protocol, DataLinkProtocol::Ethernet);
        assert!(data_link.ethernet.is_some());

        let network = capture.network.unwrap();
        assert_eq!(network.protocol, NetworkProtocol::Ipv4);
        assert_eq!(network.ipv4.unwrap().source, Ipv4Addr::new(192, 168, 1, 79));

        let transport = capture.transport.unwrap();
        assert_eq!(transport.protocol, TransportProtocol::Udp);
        assert!(transport.udp.is_some());

        let application = capture.application.unwrap();
        assert_eq!(application.protocol, ApplicationProtocol::Dns);
        assert_eq!(application.dns.unwrap().question.qname, "taivem.com");
    }
}
