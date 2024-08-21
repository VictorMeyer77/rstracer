use crate::error::Error;
use crate::osi::Layer;
use pnet::packet::ethernet::{Ethernet, EthernetPacket};
use pnet::packet::FromPacket;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DataLinkProtocol {
    Ethernet,
}

#[derive(Debug, Clone)]
pub struct DataLink {
    pub protocol: DataLinkProtocol,
    pub ethernet: Option<Ethernet>,
    pub payload: Vec<u8>,
}

impl fmt::Display for DataLinkProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DataLinkProtocol::Ethernet => "ethernet",
            }
        )
    }
}

pub fn read_packet(packet: &[u8]) -> Result<DataLink, Error> {
    if let Some(ethernet) = EthernetPacket::new(packet) {
        Ok(DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(ethernet.from_packet()),
            payload: vec![],
        })
    } else {
        Err(Error::UnimplementedError {
            layer: Layer::DataLink.to_string(),
            protocol: "".to_string(),
            data: packet.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osi::tests::create_ethernet_packet;
    use pnet::packet::ethernet::EtherTypes;

    #[test]
    fn test_display_data_link_protocol() {
        let protocol = DataLinkProtocol::Ethernet;
        assert_eq!(protocol.to_string(), "ethernet");
    }

    #[test]
    fn test_read_packet_valid() {
        let packet = create_ethernet_packet(EtherTypes::Ipv4, &[0u8; 14]);
        let result = read_packet(&packet);
        assert!(result.is_ok());

        let datalink = result.unwrap();
        assert_eq!(datalink.protocol, DataLinkProtocol::Ethernet);
        assert!(datalink.ethernet.is_some());
        assert_eq!(datalink.payload, vec![]);
    }

    #[test]
    fn test_read_packet_invalid() {
        let packet = vec![0u8; 5];
        let result = read_packet(&packet);
        assert!(result.is_err());

        if let Err(error) = result {
            match error {
                Error::UnimplementedError {
                    layer,
                    protocol,
                    data,
                } => {
                    assert_eq!(layer, "data_link");
                    assert_eq!(protocol, "");
                    assert_eq!(data, packet);
                }
                _ => panic!("Unexpected error type"),
            }
        }
    }
}
