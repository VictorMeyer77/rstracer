use crate::capture::network::{Network, NetworkProtocol};
use crate::capture::Layer;
use crate::error::Error;
use pnet::packet::icmp::{Icmp, IcmpPacket};
use pnet::packet::icmpv6::{Icmpv6, Icmpv6Packet};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::tcp::{Tcp, TcpPacket};
use pnet::packet::udp::{Udp, UdpPacket};
use pnet::packet::FromPacket;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Icmpv4, // OSI Network but process as Transport
    Icmpv6, // OSI Network but process as Transport
}

#[derive(Debug, Clone)]
pub struct Transport {
    pub protocol: TransportProtocol,
    pub tcp: Option<Tcp>,
    pub udp: Option<Udp>,
    pub icmpv4: Option<Icmp>,
    pub icmpv6: Option<Icmpv6>,
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TransportProtocol::Tcp => "tcp",
                TransportProtocol::Udp => "udp",
                TransportProtocol::Icmpv4 => "icmpv4",
                TransportProtocol::Icmpv6 => "icmpv6",
            }
        )
    }
}

impl Transport {
    fn tcp(tcp: Tcp) -> Transport {
        Transport {
            protocol: TransportProtocol::Tcp,
            tcp: Some(tcp),
            udp: None,
            icmpv4: None,
            icmpv6: None,
        }
    }

    fn udp(udp: Udp) -> Transport {
        Transport {
            protocol: TransportProtocol::Udp,
            tcp: None,
            udp: Some(udp),
            icmpv4: None,
            icmpv6: None,
        }
    }

    fn icmpv4(icmpv4: Icmp) -> Transport {
        Transport {
            protocol: TransportProtocol::Icmpv4,
            tcp: None,
            udp: None,
            icmpv4: Some(icmpv4),
            icmpv6: None,
        }
    }

    fn icmpv6(icmpv6: Icmpv6) -> Transport {
        Transport {
            protocol: TransportProtocol::Icmpv6,
            tcp: None,
            udp: None,
            icmpv4: None,
            icmpv6: Some(icmpv6),
        }
    }
}

fn parse_tcp(packet: &[u8]) -> Option<Transport> {
    TcpPacket::new(packet).map(|tcp| Transport::tcp(tcp.from_packet()))
}

fn parse_udp(packet: &[u8]) -> Option<Transport> {
    UdpPacket::new(packet).map(|udp| Transport::udp(udp.from_packet()))
}

fn parse_icmpv4(packet: &[u8]) -> Option<Transport> {
    IcmpPacket::new(packet).map(|icmpv4| Transport::icmpv4(icmpv4.from_packet()))
}

fn parse_icmpv6(packet: &[u8]) -> Option<Transport> {
    Icmpv6Packet::new(packet).map(|icmpv6| Transport::icmpv6(icmpv6.from_packet()))
}

pub fn read_packet(network: &Network) -> Result<Transport, Error> {
    match &network.protocol {
        NetworkProtocol::Ipv4 => {
            let ipv4 = network.ipv4.clone().unwrap();
            match ipv4.next_level_protocol {
                IpNextHeaderProtocols::Tcp => parse_tcp(&ipv4.payload).ok_or(Error::PacketParsing),
                IpNextHeaderProtocols::Udp => parse_udp(&ipv4.payload).ok_or(Error::PacketParsing),
                IpNextHeaderProtocols::Icmp => {
                    parse_icmpv4(&ipv4.payload).ok_or(Error::PacketParsing)
                }
                unimplemented => Err(Error::UnimplementedError {
                    layer: Layer::Transport.to_string(),
                    protocol: unimplemented.to_string(),
                }),
            }
        }
        NetworkProtocol::Ipv6 => {
            let ipv6 = network.ipv6.clone().unwrap();
            match ipv6.next_header {
                IpNextHeaderProtocols::Tcp => parse_tcp(&ipv6.payload).ok_or(Error::PacketParsing),
                IpNextHeaderProtocols::Udp => parse_udp(&ipv6.payload).ok_or(Error::PacketParsing),
                IpNextHeaderProtocols::Icmpv6 => {
                    parse_icmpv6(&ipv6.payload).ok_or(Error::PacketParsing)
                }
                unimplemented => Err(Error::UnimplementedError {
                    layer: Layer::Transport.to_string(),
                    protocol: unimplemented.to_string(),
                }),
            }
        }
        unimplemented => Err(Error::UnimplementedError {
            layer: Layer::Transport.to_string(),
            protocol: unimplemented.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::tests::{
        create_ethernet_packet, create_icmpv4_packet, create_icmpv6_packet, create_ipv4_packet,
        create_ipv6_packet, create_tcp_packet, create_udp_packet,
    };
    use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
    use pnet::packet::ethernet::EtherTypes;
    use pnet::packet::ip::IpNextHeaderProtocols;
    use pnet::packet::ipv4::Ipv4Packet;
    use pnet::packet::ipv6::Ipv6Packet;
    use pnet::packet::udp::MutableUdpPacket;
    use pnet::packet::Packet;
    use pnet::util::MacAddr;
    use std::net::Ipv4Addr;

    #[test]
    fn test_parse_tcp_valid() {
        let payload = b"valid tcp";
        let packet = create_tcp_packet(12345, 80, payload);
        let ipv4_packet = Ipv4Packet::new(&packet[14..]).unwrap();

        let transport = parse_tcp(ipv4_packet.payload()).unwrap();
        assert_eq!(transport.protocol, TransportProtocol::Tcp);

        let tcp = transport.tcp.unwrap();
        assert_eq!(tcp.source, 12345);
        assert_eq!(tcp.destination, 80);
    }

    #[test]
    fn test_parse_tcp_invalid() {
        let invalid_payload = b"";
        let result = parse_tcp(invalid_payload);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_udp_valid() {
        let payload = b"valid udp";
        let packet = create_udp_packet(54321, 53, payload);
        let ipv4_packet = Ipv4Packet::new(&packet[14..]).unwrap();

        let transport = parse_udp(ipv4_packet.payload()).unwrap();
        assert_eq!(transport.protocol, TransportProtocol::Udp);

        let udp = transport.udp.unwrap();
        assert_eq!(udp.source, 54321);
        assert_eq!(udp.destination, 53);
    }

    #[test]
    fn test_parse_udp_invalid() {
        let invalid_payload = b"";
        let result = parse_udp(invalid_payload);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_icmpv4_valid() {
        let payload = b"valid icmpv4";
        let packet = create_icmpv4_packet(payload);
        let ipv4_packet = Ipv4Packet::new(&packet[14..]).unwrap();

        let transport = parse_icmpv4(ipv4_packet.payload()).unwrap();
        assert_eq!(transport.protocol, TransportProtocol::Icmpv4);
        assert!(transport.icmpv4.is_some());
    }

    #[test]
    fn test_parse_icmpv4_invalid() {
        let invalid_payload = b"";
        let result = parse_icmpv4(invalid_payload);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_icmpv6_valid() {
        let payload = b"valid icmpv6";
        let packet = create_icmpv6_packet(payload);
        let ipv6_packet = Ipv6Packet::new(&packet[14..]).unwrap();

        let transport = parse_icmpv6(ipv6_packet.payload()).unwrap();
        assert_eq!(transport.protocol, TransportProtocol::Icmpv6);
        assert!(transport.icmpv6.is_some());
    }

    #[test]
    fn test_parse_icmpv6_invalid() {
        let invalid_payload = b"";
        let result = parse_icmpv6(invalid_payload);
        assert!(result.is_none());
    }

    #[test]
    fn test_read_packet_ipv4_tcp() {
        let payload = b"valid ipv4 tcp";
        let ethernet_packet = create_tcp_packet(1234, 80, payload);
        let network = Network {
            protocol: NetworkProtocol::Ipv4,
            ipv4: Some(
                Ipv4Packet::new(&ethernet_packet[14..])
                    .unwrap()
                    .from_packet(),
            ),
            ipv6: None,
            arp: None,
        };

        let result = read_packet(&network);
        assert!(result.is_ok());

        let transport = result.unwrap();
        assert_eq!(transport.protocol, TransportProtocol::Tcp);
        assert!(transport.tcp.is_some());

        let tcp = transport.tcp.unwrap();
        assert_eq!(tcp.source, 1234);
        assert_eq!(tcp.destination, 80);
    }

    #[test]
    fn test_read_packet_ipv4_unsupported() {
        let payload = b"";
        let ipv4_packet = create_ipv4_packet(IpNextHeaderProtocols::Igmp, payload);
        let network = Network {
            protocol: NetworkProtocol::Ipv4,
            ipv4: Some(Ipv4Packet::new(&ipv4_packet[14..]).unwrap().from_packet()),
            ipv6: None,
            arp: None,
        };

        let result = read_packet(&network);
        assert!(result.is_err());

        let error = result.err().unwrap();
        match error {
            Error::UnimplementedError { layer, protocol } => {
                assert_eq!(layer, Layer::Transport.to_string());
                assert_eq!(protocol, IpNextHeaderProtocols::Igmp.to_string());
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_read_packet_ipv6_unsupported() {
        let payload = b"";
        let ipv6_packet = create_ipv6_packet(IpNextHeaderProtocols::Igmp, payload);
        let network = Network {
            protocol: NetworkProtocol::Ipv6,
            ipv4: None,
            ipv6: Some(Ipv6Packet::new(&ipv6_packet[14..]).unwrap().from_packet()),
            arp: None,
        };

        let result = read_packet(&network);
        assert!(result.is_err());

        let error = result.err().unwrap();
        match error {
            Error::UnimplementedError { layer, protocol } => {
                assert_eq!(layer, Layer::Transport.to_string());
                assert_eq!(protocol, IpNextHeaderProtocols::Igmp.to_string());
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_read_packet_ipv6_udp() {
        let payload = b"valid ipv6 udp";
        let udp_packet_data = {
            let mut udp_packet = MutableUdpPacket::owned(vec![0u8; 8 + payload.len()]).unwrap();
            udp_packet.set_source(1234);
            udp_packet.set_destination(53);
            udp_packet.set_length((8 + payload.len()) as u16);
            udp_packet.set_checksum(0);
            udp_packet.set_payload(payload);
            udp_packet.packet().to_vec()
        };
        let ethernet_packet = create_ipv6_packet(IpNextHeaderProtocols::Udp, &udp_packet_data);
        let network = Network {
            protocol: NetworkProtocol::Ipv6,
            ipv4: None,
            ipv6: Some(
                Ipv6Packet::new(&ethernet_packet[14..])
                    .unwrap()
                    .from_packet(),
            ),
            arp: None,
        };

        let result = read_packet(&network);
        assert!(result.is_ok());

        let transport = result.unwrap();
        assert_eq!(transport.protocol, TransportProtocol::Udp);
        assert!(transport.udp.is_some());

        let udp = transport.udp.unwrap();
        assert_eq!(udp.source, 1234);
        assert_eq!(udp.destination, 53);
    }

    #[test]
    fn test_read_packet_arp() {
        let arp_packet_data = {
            let mut arp_packet = MutableArpPacket::owned(vec![0u8; 28]).unwrap();
            arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
            arp_packet.set_protocol_type(EtherTypes::Ipv4);
            arp_packet.set_operation(ArpOperations::Request);
            arp_packet.set_sender_hw_addr(MacAddr::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55));
            arp_packet.set_sender_proto_addr(Ipv4Addr::new(192, 168, 0, 1));
            arp_packet.set_target_hw_addr(MacAddr::new(0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb));
            arp_packet.set_target_proto_addr(Ipv4Addr::new(192, 168, 0, 2));
            arp_packet.packet().to_vec()
        };
        let ethernet_packet = create_ethernet_packet(EtherTypes::Arp, &arp_packet_data);
        let network = Network {
            protocol: NetworkProtocol::Arp,
            ipv4: None,
            ipv6: None,
            arp: Some(
                ArpPacket::new(&ethernet_packet[14..])
                    .unwrap()
                    .from_packet(),
            ),
        };

        let result = read_packet(&network);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert_eq!(
            error.to_string(),
            "Protocol arp on layer transport is not implemented yet"
        )
    }
}
