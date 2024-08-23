use crate::capture::transport::{Transport, TransportProtocol};
use crate::capture::Layer;
use crate::error::Error;
use pnet::packet::dns::{Dns, DnsPacket};
use pnet::packet::FromPacket;
use std::fmt;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum ApplicationProtocol {
    Dns,
    // TODO FTP, HTTP, HTTPS etc
}

#[derive(Debug, Clone)]
pub struct Application {
    pub protocol: ApplicationProtocol,
    pub dns: Option<Dns>,
    pub payload: Vec<u8>,
}

impl fmt::Display for ApplicationProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ApplicationProtocol::Dns => "dns",
            }
        )
    }
}

impl Application {
    fn dns(dns: Dns) -> Application {
        Application {
            protocol: ApplicationProtocol::Dns,
            dns: Some(dns),
            payload: vec![],
        }
    }
}

fn parse_dns(packet: &[u8]) -> Result<Application, Error> {
    if let Some(dns) = DnsPacket::new(packet) {
        Ok(Application::dns(dns.from_packet()))
    } else {
        Err(Error::PacketParseError {
            layer: Layer::Application.to_string(),
            protocol: ApplicationProtocol::Dns.to_string(),
            data: packet.to_vec(),
        })
    }
}

pub fn read_packet(transport: &Transport) -> Result<Application, Error> {
    let transport = transport.clone();

    match transport.protocol {
        TransportProtocol::Tcp => parse_dns(&transport.tcp.clone().unwrap().payload), // todo
        TransportProtocol::Udp => parse_dns(&transport.udp.clone().unwrap().payload),
        unimplemented => Err(Error::UnimplementedError {
            layer: Layer::Application.to_string(),
            protocol: format!("{}", unimplemented).to_lowercase(),
        }),
    }
}

// todo test
