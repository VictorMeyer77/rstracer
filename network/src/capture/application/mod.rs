use crate::capture::application::http::Http;
use crate::capture::application::tls::Tls;
use crate::capture::transport::{Transport, TransportProtocol};
use crate::capture::Layer;
use crate::error::Error;
use pnet::packet::dns::Dns;
use std::fmt;

mod dns;
pub mod http;
pub mod tls;

#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationProtocol {
    Dns,
    Http,
    Tls,
}

#[derive(Debug, Clone)]
pub struct Application {
    pub protocol: ApplicationProtocol,
    pub dns: Option<Dns>,
    pub http: Option<Http>,
    pub tls: Option<Tls>,
}

impl fmt::Display for ApplicationProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ApplicationProtocol::Dns => "dns",
                ApplicationProtocol::Http => "http",
                ApplicationProtocol::Tls => "tls",
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
            tls: None,
        }
    }

    fn http(http: Http) -> Application {
        Application {
            protocol: ApplicationProtocol::Http,
            dns: None,
            http: Some(http),
            tls: None,
        }
    }

    fn tls(tls: Tls) -> Application {
        Application {
            protocol: ApplicationProtocol::Tls,
            dns: None,
            http: None,
            tls: Some(tls),
        }
    }
}

fn parse_dns(packet: &[u8]) -> Option<Application> {
    dns::from_bytes(packet).map(Application::dns)
}

fn parse_http(packet: &[u8]) -> Option<Application> {
    if let Ok(http) = Http::from_bytes(packet) {
        Some(Application::http(http))
    } else {
        None
    }
}

fn parse_tls(packet: &[u8]) -> Option<Application> {
    if let Ok(tls) = Tls::from_bytes(packet) {
        Some(Application::tls(tls))
    } else {
        None
    }
}

fn parse_tcp(packet: &[u8]) -> Option<Application> {
    let mut application = parse_dns(packet);
    if application.is_none() {
        application = parse_http(packet)
    }
    if application.is_none() {
        application = parse_tls(packet)
    }
    application
}

fn parse_udp(packet: &[u8]) -> Option<Application> {
    parse_dns(packet)
}

pub fn read_packet(transport: &Transport) -> Result<Application, Error> {
    let transport = transport.clone();
    match transport.protocol {
        TransportProtocol::Tcp => {
            parse_tcp(&transport.tcp.unwrap().payload).ok_or(Error::PacketParsing)
        }
        TransportProtocol::Udp => {
            parse_udp(&transport.udp.unwrap().payload).ok_or(Error::PacketParsing)
        }
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
    use crate::capture::application::tls::TlsContentType;
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

        let application = read_packet(&transport).unwrap();
        let dns = application.dns.unwrap();

        assert_eq!(application.protocol, ApplicationProtocol::Dns);
        assert_eq!(&dns.responses.first().unwrap().data, &[93, 184, 216, 34])
    }

    #[test]
    fn test_read_packet_udp_dns() {
        let dns_bytes = create_dns_packet();
        let transport = create_mock_udp_transport(&dns_bytes);

        let application = read_packet(&transport).unwrap();
        let dns = application.dns.unwrap();

        assert_eq!(application.protocol, ApplicationProtocol::Dns);
        assert_eq!(&dns.responses.first().unwrap().data, &[93, 184, 216, 34])
    }

    #[test]
    fn test_read_packet_tcp_http() {
        let http_bytes = create_http_packet();
        let transport = create_mock_tcp_transport(&http_bytes);

        let application = read_packet(&transport).unwrap();
        assert_eq!(application.protocol, ApplicationProtocol::Http);
        assert_eq!(application.http.unwrap().body, "name=ChatGPT&language=Rust");
    }

    #[test]
    fn test_read_packet_tcp_tls() {
        let tls_bytes = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x02, 0x03, 0x04, 0x05];
        let transport = create_mock_tcp_transport(&tls_bytes);

        let application = read_packet(&transport).unwrap();

        assert_eq!(application.protocol, ApplicationProtocol::Tls);
        assert_eq!(
            application.tls.unwrap().content_type,
            TlsContentType::Handshake
        );
    }
}
