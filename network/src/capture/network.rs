use crate::capture::data_link::{DataLink, DataLinkProtocol};
use crate::capture::Layer;
use crate::error::Error;
use pnet::packet::arp::{Arp, ArpPacket};
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ipv4::{Ipv4, Ipv4Packet};
use pnet::packet::ipv6::{Ipv6, Ipv6Packet};
use pnet::packet::FromPacket;
use std::fmt;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum NetworkProtocol {
    Arp,
    Ipv4,
    Ipv6,
}

#[derive(Debug, Clone)]
pub struct Network {
    pub protocol: NetworkProtocol,
    pub ipv4: Option<Ipv4>,
    pub ipv6: Option<Ipv6>,
    pub arp: Option<Arp>,
}

impl fmt::Display for NetworkProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NetworkProtocol::Arp => "arp",
                NetworkProtocol::Ipv4 => "ipv4",
                NetworkProtocol::Ipv6 => "ipv6",
            }
        )
    }
}

impl Network {
    fn arp(arp: Arp) -> Network {
        Network {
            protocol: NetworkProtocol::Arp,
            arp: Some(arp),
            ipv4: None,
            ipv6: None,
        }
    }

    fn ipv4(ipv4: Ipv4) -> Network {
        Network {
            protocol: NetworkProtocol::Ipv4,
            ipv4: Some(ipv4),
            arp: None,
            ipv6: None,
        }
    }

    fn ipv6(ipv6: Ipv6) -> Network {
        Network {
            protocol: NetworkProtocol::Ipv6,
            ipv4: None,
            arp: None,
            ipv6: Some(ipv6),
        }
    }
}

fn parse_arp(packet: &[u8]) -> Result<Network, Error> {
    if let Some(arp) = ArpPacket::new(packet) {
        Ok(Network::arp(arp.from_packet()))
    } else {
        Err(Error::PacketParseError {
            layer: Layer::Network.to_string(),
            protocol: NetworkProtocol::Arp.to_string(),
            data: packet.to_vec(),
        })
    }
}

fn parse_ipv4(packet: &[u8]) -> Result<Network, Error> {
    if let Some(ipv4) = Ipv4Packet::new(packet) {
        Ok(Network::ipv4(ipv4.from_packet()))
    } else {
        Err(Error::PacketParseError {
            layer: Layer::Network.to_string(),
            protocol: NetworkProtocol::Ipv4.to_string(),
            data: packet.to_vec(),
        })
    }
}

fn parse_ipv6(packet: &[u8]) -> Result<Network, Error> {
    if let Some(ipv6) = Ipv6Packet::new(packet) {
        Ok(Network::ipv6(ipv6.from_packet()))
    } else {
        Err(Error::PacketParseError {
            layer: Layer::Network.to_string(),
            protocol: NetworkProtocol::Ipv6.to_string(),
            data: packet.to_vec(),
        })
    }
}

pub fn read_packet(data_link: &DataLink) -> Result<Network, Error> {
    match data_link.protocol {
        DataLinkProtocol::Ethernet => {
            let ethernet = data_link.ethernet.clone().unwrap();
            match ethernet.ethertype {
                EtherTypes::Ipv4 => parse_ipv4(&ethernet.payload),
                EtherTypes::Ipv6 => parse_ipv6(&ethernet.payload),
                EtherTypes::Arp => parse_arp(&ethernet.payload),
                unimplemented => Err(Error::UnimplementedError {
                    layer: Layer::Network.to_string(),
                    protocol: format!("{}", unimplemented).to_lowercase(),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::tests::{create_arp_packet, create_ipv4_packet, create_ipv6_packet};
    use pnet::packet::ethernet::MutableEthernetPacket;
    use pnet::packet::ip::IpNextHeaderProtocols;

    #[test]
    fn test_display_network_protocol() {
        assert_eq!(NetworkProtocol::Arp.to_string(), "arp");
        assert_eq!(NetworkProtocol::Ipv4.to_string(), "ipv4");
        assert_eq!(NetworkProtocol::Ipv6.to_string(), "ipv6");
    }

    #[test]
    fn test_parse_arp_valid() {
        let frame = create_arp_packet();
        let result = parse_arp(&frame);
        assert!(result.is_ok());

        let network = result.unwrap();
        assert_eq!(network.protocol, NetworkProtocol::Arp);
        assert!(network.arp.is_some());
        assert!(network.ipv4.is_none());
        assert!(network.ipv6.is_none());
    }

    #[test]
    fn test_parse_arp_invalid() {
        let invalid_payload = b"";
        let result = parse_arp(invalid_payload);
        assert!(result.is_err());

        let error = result.err().unwrap();
        match error {
            Error::PacketParseError {
                layer,
                protocol,
                data,
            } => {
                assert_eq!(layer, Layer::Network.to_string());
                assert_eq!(protocol, NetworkProtocol::Arp.to_string());
                assert_eq!(data, invalid_payload.to_vec());
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_parse_ipv4_valid() {
        let frame = create_ipv4_packet(IpNextHeaderProtocols::Icmp, &[0u8; 20]);

        let result = parse_ipv4(&frame);
        assert!(result.is_ok());

        let network = result.unwrap();
        assert_eq!(network.protocol, NetworkProtocol::Ipv4);
        assert!(network.ipv4.is_some());
        assert!(network.arp.is_none());
        assert!(network.ipv6.is_none());
    }

    #[test]
    fn test_parse_icpv4_invalid() {
        let invalid_payload = b"";
        let result = parse_ipv4(invalid_payload);
        assert!(result.is_err());

        let error = result.err().unwrap();
        match error {
            Error::PacketParseError {
                layer,
                protocol,
                data,
            } => {
                assert_eq!(layer, Layer::Network.to_string());
                assert_eq!(protocol, NetworkProtocol::Ipv4.to_string());
                assert_eq!(data, invalid_payload.to_vec());
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_parse_ipv6_valid() {
        let frame = create_ipv6_packet(IpNextHeaderProtocols::Udp, &[0u8; 40]);
        let result = parse_ipv6(&frame);
        assert!(result.is_ok());

        let network = result.unwrap();
        assert_eq!(network.protocol, NetworkProtocol::Ipv6);
        assert!(network.ipv6.is_some());
        assert!(network.arp.is_none());
        assert!(network.ipv4.is_none());
    }

    #[test]
    fn test_parse_icpv6_invalid() {
        let invalid_payload = b"";
        let result = parse_ipv6(invalid_payload);
        assert!(result.is_err());

        let error = result.err().unwrap();
        match error {
            Error::PacketParseError {
                layer,
                protocol,
                data,
            } => {
                assert_eq!(layer, Layer::Network.to_string());
                assert_eq!(protocol, NetworkProtocol::Ipv6.to_string());
                assert_eq!(data, invalid_payload.to_vec());
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_read_packet_ethernet_ipv4() {
        let ethernet_packet = MutableEthernetPacket::owned(create_ipv4_packet(
            IpNextHeaderProtocols::Icmp,
            &[0u8; 20],
        ))
        .unwrap();
        let data_link = DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(ethernet_packet.from_packet()),
        };
        let result = read_packet(&data_link);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().protocol, NetworkProtocol::Ipv4);
    }

    #[test]
    fn test_read_packet_ethernet_ipv6() {
        let ethernet_packet = MutableEthernetPacket::owned(create_ipv6_packet(
            IpNextHeaderProtocols::Udp,
            &[0u8; 40],
        ))
        .unwrap();
        let data_link = DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(ethernet_packet.from_packet()),
        };
        let result = read_packet(&data_link);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().protocol, NetworkProtocol::Ipv6);
    }

    #[test]
    fn test_read_packet_ethernet_arp() {
        let ethernet_packet = MutableEthernetPacket::owned(create_arp_packet()).unwrap();
        let data_link = DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(ethernet_packet.from_packet()),
        };
        let result = read_packet(&data_link);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().protocol, NetworkProtocol::Arp);
    }

    #[test]
    fn test_read_packet_unimplemented_protocol() {
        let ethernet_packet = MutableEthernetPacket::owned(vec![0u8; 14]).unwrap();
        let data_link = DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(ethernet_packet.from_packet()),
        };
        let result = read_packet(&data_link);
        assert!(result.is_err());

        if let Err(Error::UnimplementedError { layer, protocol }) = result {
            assert_eq!(layer, "network");
            assert_eq!(protocol, "unknown");
        } else {
            panic!("Expected UnimplementedError");
        }
    }
}
