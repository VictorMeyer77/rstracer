use crate::error::Error;
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

pub fn read_packet(packet: &[u8]) -> Result<DataLink, Error> {
    if let Some(ethernet) = EthernetPacket::new(&packet) {
        Ok(DataLink {
            protocol: DataLinkProtocol::Ethernet,
            ethernet: Some(Ethernet {
                source: ethernet.get_source().to_string(),
                destination: ethernet.get_destination().to_string(),
                ether_type: ethernet.get_ethertype(),
            }),
            payload: ethernet.payload().to_vec(),
        })
    } else {
        Ok(DataLink {
            protocol: DataLinkProtocol::Unknown,
            ethernet: None,
            payload: packet.to_vec(),
        })
    }
}
