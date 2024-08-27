use crate::capture::application::dns::Dns;
use crate::capture::application::http::Http;
use crate::capture::transport::{Transport, TransportProtocol};
use crate::capture::Layer;
use crate::error::Error;
use std::fmt;

pub mod dns;
pub mod http;

#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationProtocol {
    Dns,
    Http,
}

#[derive(Debug, Clone)]
pub struct Application {
    pub protocol: ApplicationProtocol,
    pub dns: Option<Dns>,
    pub http: Option<Http>,
}

impl fmt::Display for ApplicationProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ApplicationProtocol::Dns => "dns",
                ApplicationProtocol::Http => "http",
            }
        )
    }
}

impl Application {
    fn dns(dns: Dns) -> Application {
        Application {
            protocol: ApplicationProtocol::Dns,
            dns: Some(dns),
            http: None,
        }
    }

    fn http(http: Http) -> Application {
        Application {
            protocol: ApplicationProtocol::Http,
            dns: None,
            http: Some(http),
        }
    }
}

fn parse_dns(packet: &[u8]) -> Result<Application, Error> {
    Ok(Application::dns(Dns::from_bytes(packet)?))
}

fn parse_http(packet: &[u8]) -> Result<Application, Error> {
    Ok(Application::http(Http::from_bytes(packet)?))
}

fn parse_tcp(packet: &[u8]) -> Result<Application, Error> {
    let application = parse_dns(packet);
    if application.is_ok() {
        return application;
    }
    let application = parse_http(packet);
    if application.is_ok() {
        return application;
    }
    Err(Error::PacketParseError {
        layer: Layer::Application.to_string(),
        protocol: "".to_string(),
        data: packet.to_vec(),
    })
}

fn parse_udp(packet: &[u8]) -> Result<Application, Error> {
    let application = parse_dns(packet);
    if application.is_ok() {
        return application;
    }
    Err(Error::PacketParseError {
        layer: Layer::Application.to_string(),
        protocol: "".to_string(),
        data: packet.to_vec(),
    })
}

pub fn read_packet(transport: &Transport) -> Result<Application, Error> {
    let transport = transport.clone();
    match transport.protocol {
        TransportProtocol::Tcp => parse_tcp(&transport.tcp.unwrap().payload),
        TransportProtocol::Udp => parse_udp(&transport.udp.unwrap().payload),
        unimplemented => Err(Error::UnimplementedError {
            layer: Layer::Application.to_string(),
            protocol: format!("{}", unimplemented).to_lowercase(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::application::dns::tests::create_dns_packet;
    use crate::capture::application::http::tests::create_http_packet;
    use pnet::packet::tcp::Tcp;
    use pnet::packet::udp::Udp;

    fn create_mock_tcp_transport(payload: &[u8]) -> Transport {
        let tcp = Tcp {
            source: 1247,
            destination: 53,
            sequence: 0,
            acknowledgement: 0,
            data_offset: 0,
            reserved: 0,
            flags: 0,
            window: 0,
            checksum: 0,
            urgent_ptr: 0,
            options: vec![],
            payload: payload.to_vec(),
        };
        Transport {
            protocol: TransportProtocol::Tcp,
            tcp: Some(tcp),
            udp: None,
            icmpv4: None,
            icmpv6: None,
        }
    }

    fn create_mock_udp_transport(payload: &[u8]) -> Transport {
        let udp = Udp {
            source: 8646,
            destination: 53,
            length: 0,
            checksum: 0,
            payload: payload.to_vec(),
        };
        Transport {
            protocol: TransportProtocol::Udp,
            tcp: None,
            udp: Some(udp),
            icmpv4: None,
            icmpv6: None,
        }
    }

    #[test]
    fn test_display_application_protocol() {
        assert_eq!(ApplicationProtocol::Dns.to_string(), "dns");
    }

    #[test]
    fn test_read_packet_tcp_dns() {
        let dns_bytes = create_dns_packet();
        let transport = create_mock_tcp_transport(&dns_bytes);

        let result = read_packet(&transport);
        assert!(result.is_ok());

        let application = result.unwrap();
        assert_eq!(application.protocol, ApplicationProtocol::Dns);
        assert_eq!(application.dns.unwrap().question.qname, "www.example.com");
    }

    #[test]
    fn test_read_packet_udp_dns() {
        let dns_bytes = create_dns_packet();
        let transport = create_mock_udp_transport(&dns_bytes);

        let result = read_packet(&transport);
        assert!(result.is_ok());

        let application = result.unwrap();
        assert_eq!(application.protocol, ApplicationProtocol::Dns);
        assert_eq!(application.dns.unwrap().question.qname, "www.example.com");
    }

    #[test]
    fn test_read_packet_tcp_http() {
        let http_bytes = create_http_packet();
        let transport = create_mock_tcp_transport(&http_bytes);

        let result = read_packet(&transport);
        assert!(result.is_ok());

        let application = result.unwrap();
        assert_eq!(application.protocol, ApplicationProtocol::Http);
        assert_eq!(application.http.unwrap().body, "name=ChatGPT&language=Rust");
    }
}
