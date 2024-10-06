use lsof::lsof::OpenFile;
use network::capture::application::http::Http;
use network::capture::application::tls::Tls;
use network::capture::application::{Application, ApplicationProtocol};
use network::capture::data_link::{DataLink, DataLinkProtocol};
use network::capture::network::{Network, NetworkProtocol};
use network::capture::transport::{Transport, TransportProtocol};
use network::capture::Capture;
use pcap::Device;
use pnet::packet::arp::Arp;
use pnet::packet::dns::Dns;
use pnet::packet::ethernet::Ethernet;
use pnet::packet::icmp::Icmp;
use pnet::packet::icmpv6::Icmpv6;
use pnet::packet::ipv4::Ipv4;
use pnet::packet::ipv6::Ipv6;
use pnet::packet::tcp::Tcp;
use pnet::packet::udp::Udp;
use pnet::packet::PrimitiveValues;
use ps::ps::Process;
use uuid::Uuid;

pub trait Bronze {
    fn to_insert_sql(&self, foreign_id: Option<u128>) -> String;
}

pub trait BronzeBatch {
    fn get_insert_header() -> String;

    fn to_insert_value(&self) -> String;
}

impl BronzeBatch for Process {
    fn get_insert_header() -> String {
        r#"INSERT INTO memory.bronze_process_list
        (pid, ppid, uid, lstart, pcpu, pmem, status, command, created_at, inserted_at, brz_ingestion_duration)
        VALUES "#
            .to_string()
    }

    fn to_insert_value(&self) -> String {
        format!("({}, {}, {}, TO_TIMESTAMP({}), {}, {}, '{}', '{}', TO_TIMESTAMP({8}), CURRENT_TIMESTAMP, AGE(TO_TIMESTAMP({8})::TIMESTAMP))",
            self.pid,
            self.ppid,
            self.uid,
            self.lstart,
            self.pcpu,
            self.pmem,
            self.status,
            self.command.replace('\'', "\""),
            self.created_at
        )
    }
}

impl BronzeBatch for OpenFile {
    fn get_insert_header() -> String {
        r#"INSERT INTO memory.bronze_open_files
        (command, pid, uid, fd, type, device, size, node, name, created_at, inserted_at, brz_ingestion_duration)
        VALUES "#
            .to_string()
    }

    fn to_insert_value(&self) -> String {
        format!(
            r#"('{}', {}, {}, '{}', '{}', '{}', {}, '{}', '{}', TO_TIMESTAMP({9}), CURRENT_TIMESTAMP, AGE(TO_TIMESTAMP({9})::TIMESTAMP))"#,
            self.command.replace('\'', "\""),
            self.pid,
            self.uid,
            self.fd,
            self._type,
            self.device,
            self.size,
            self.node,
            self.name.replace('\'', "\""),
            self.created_at
        )
    }
}

impl Bronze for Capture {
    fn to_insert_sql(&self, _foreign_id: Option<u128>) -> String {
        let clone = self.clone();
        let row_id = Uuid::new_v4().as_u128();
        let mut request_buffer = format!(
            r#"INSERT OR REPLACE INTO memory.bronze_network_packet (
            _id,
            interface,
            length,
            created_at,
            inserted_at,
            brz_ingestion_duration
            ) VALUES ({}, '{}', {}, TO_TIMESTAMP({3}), CURRENT_TIMESTAMP, AGE(TO_TIMESTAMP({3})::TIMESTAMP));"#,
            row_id,
            clone.device.name,
            clone.packet.len(),
            clone.created_at
        );

        request_buffer.push_str(&device_addresses_to_sql(&clone.device));

        if clone.data_link.is_some() {
            request_buffer.push_str(&clone.data_link.unwrap().to_insert_sql(Some(row_id)))
        }

        if clone.network.is_some() {
            request_buffer.push_str(&clone.network.unwrap().to_insert_sql(Some(row_id)))
        }

        if clone.transport.is_some() {
            request_buffer.push_str(&clone.transport.unwrap().to_insert_sql(Some(row_id)))
        }

        if clone.application.is_some() {
            request_buffer.push_str(&clone.application.unwrap().to_insert_sql(Some(row_id)))
        }

        request_buffer
    }
}

fn device_addresses_to_sql(device: &Device) -> String {
    let mut request_buffer = String::new();

    for address in &device.addresses {
        let netmask = if let Some(netmask) = address.netmask {
            netmask.to_string()
        } else {
            "NULL".to_string()
        };
        let broadcast_address = if let Some(broadcast_address) = address.broadcast_addr {
            broadcast_address.to_string()
        } else {
            "NULL".to_string()
        };
        let destination_address = if let Some(destination_address) = address.dst_addr {
            destination_address.to_string()
        } else {
            "NULL".to_string()
        };

        request_buffer.push_str(&format!(
            r#"INSERT OR IGNORE INTO memory.bronze_network_interface_address (
                        interface,
                        address,
                        netmask,
                        broadcast_address,
                        destination_addrss,
                        inserted_at
                    )
                    VALUES ('{}', '{}', '{}', '{}', {}, CURRENT_TIMESTAMP);"#,
            device.name, address.addr, netmask, broadcast_address, destination_address
        ));
    }

    request_buffer
}

impl Bronze for DataLink {
    fn to_insert_sql(&self, foreign_id: Option<u128>) -> String {
        match self.protocol {
            DataLinkProtocol::Ethernet => {
                bronze_ethernet(self.ethernet.clone().unwrap(), foreign_id.unwrap())
            }
        }
    }
}

impl Bronze for Network {
    fn to_insert_sql(&self, foreign_id: Option<u128>) -> String {
        match self.protocol {
            NetworkProtocol::Arp => bronze_arp(self.arp.clone().unwrap(), foreign_id.unwrap()),
            NetworkProtocol::Ipv4 => bronze_ipv4(self.ipv4.clone().unwrap(), foreign_id.unwrap()),
            NetworkProtocol::Ipv6 => bronze_ipv6(self.ipv6.clone().unwrap(), foreign_id.unwrap()),
        }
    }
}

impl Bronze for Transport {
    fn to_insert_sql(&self, foreign_id: Option<u128>) -> String {
        match self.protocol {
            TransportProtocol::Tcp => bronze_tcp(self.tcp.clone().unwrap(), foreign_id.unwrap()),
            TransportProtocol::Udp => bronze_udp(self.udp.clone().unwrap(), foreign_id.unwrap()),
            TransportProtocol::Icmpv4 => {
                bronze_icmpv4(self.icmpv4.clone().unwrap(), foreign_id.unwrap())
            }
            TransportProtocol::Icmpv6 => {
                bronze_icmpv6(self.icmpv6.clone().unwrap(), foreign_id.unwrap())
            }
        }
    }
}

impl Bronze for Application {
    fn to_insert_sql(&self, foreign_id: Option<u128>) -> String {
        match self.protocol {
            ApplicationProtocol::Dns => {
                let dns = self.dns.clone().unwrap();
                let mut request_buffer = bronze_dns_header(&dns, foreign_id.unwrap());
                request_buffer.push_str(&bronze_dns_query(&dns, foreign_id.unwrap()));
                request_buffer.push_str(&bronze_dns_record(&dns, foreign_id.unwrap()));
                request_buffer
            }
            ApplicationProtocol::Http => {
                bronze_http(self.http.clone().unwrap(), foreign_id.unwrap())
            }
            ApplicationProtocol::Tls => bronze_tls(self.tls.clone().unwrap(), foreign_id.unwrap()),
        }
    }
}

fn bronze_ethernet(ethernet: Ethernet, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_ethernet (
                    packet_id,
                    source,
                    destination,
                    ether_type,
                    payload_length,
                    inserted_at
                    ) VALUES ({}, '{}', '{}', {}, {}, CURRENT_TIMESTAMP);"#,
        packet_id,
        ethernet.source,
        ethernet.destination,
        ethernet.ethertype.0,
        ethernet.payload.len()
    )
}

fn bronze_arp(arp: Arp, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_arp (
                    packet_id,
                    hardware_type,
                    protocol_type,
                    hw_addr_len,
                    proto_addr_len,
                    operation,
                    sender_hw_addr,
                    sender_proto_addr,
                    target_hw_addr,
                    target_proto_addr,
                    inserted_at
                    ) VALUES ({}, {}, {}, {}, {}, {}, '{}', '{}', '{}', '{}', CURRENT_TIMESTAMP);"#,
        packet_id,
        arp.hardware_type.0,
        arp.protocol_type.0,
        arp.hw_addr_len,
        arp.proto_addr_len,
        arp.operation.0,
        arp.sender_hw_addr,
        arp.sender_proto_addr,
        arp.target_hw_addr,
        arp.target_proto_addr
    )
}

fn bronze_ipv4(ipv4: Ipv4, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_ipv4 (
                    packet_id,
                    version,
                    header_length,
                    dscp,
                    ecn,
                    total_length,
                    identification,
                    flags,
                    fragment_offset,
                    ttl,
                    next_level_protocol,
                    checksum,
                    source,
                    destination,
                    inserted_at
                    ) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, '{}', '{}', CURRENT_TIMESTAMP);"#,
        packet_id,
        ipv4.version,
        ipv4.header_length,
        ipv4.dscp,
        ipv4.ecn,
        ipv4.total_length,
        ipv4.identification,
        ipv4.flags,
        ipv4.fragment_offset,
        ipv4.ttl,
        ipv4.next_level_protocol.0,
        ipv4.checksum,
        ipv4.source,
        ipv4.destination
    )
}

fn bronze_ipv6(ipv6: Ipv6, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_ipv6 (
                    packet_id,
                    version,
                    traffic_class,
                    flow_label,
                    payload_length,
                    next_header,
                    hop_limit,
                    source,
                    destination,
                    inserted_at
                    ) VALUES ({}, {}, {}, {}, {}, {}, {}, '{}', '{}', CURRENT_TIMESTAMP);"#,
        packet_id,
        ipv6.version,
        ipv6.traffic_class,
        ipv6.flow_label,
        ipv6.payload_length,
        ipv6.next_header.0,
        ipv6.hop_limit,
        ipv6.source,
        ipv6.destination
    )
}

fn bronze_tcp(tcp: Tcp, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_tcp (
                    packet_id,
                    source,
                    destination,
                    sequence,
                    acknowledgement,
                    data_offset,
                    reserved,
                    flags,
                    _window,
                    checksum,
                    urgent_ptr,
                    options,
                    inserted_at
                    ) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, '{:?}', CURRENT_TIMESTAMP);"#,
        packet_id,
        tcp.source,
        tcp.destination,
        tcp.sequence,
        tcp.acknowledgement,
        tcp.data_offset,
        tcp.reserved,
        tcp.flags,
        tcp.window,
        tcp.checksum,
        tcp.urgent_ptr,
        tcp.options
    )
}

fn bronze_udp(udp: Udp, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_udp (
                    packet_id,
                    source,
                    destination,
                    length,
                    checksum,
                    inserted_at
                    ) VALUES ({}, {}, {}, {}, {}, CURRENT_TIMESTAMP);"#,
        packet_id, udp.source, udp.destination, udp.length, udp.checksum
    )
}

fn bronze_icmpv4(icmpv4: Icmp, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_icmp (
                    packet_id,
                    version,
                    type,
                    code,
                    checksum,
                    payload_length,
                    inserted_at
                    ) VALUES ({}, 4, {}, {}, {}, '{}', CURRENT_TIMESTAMP);"#,
        packet_id,
        icmpv4.icmp_type.0,
        icmpv4.icmp_code.0,
        icmpv4.checksum,
        icmpv4.payload.len(),
    )
}

fn bronze_icmpv6(icmpv6: Icmpv6, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_icmp (
                    packet_id,
                    version,
                    type,
                    code,
                    checksum,
                    payload_length,
                    inserted_at
                    ) VALUES ({}, 6, {}, {}, {}, {}, CURRENT_TIMESTAMP);"#,
        packet_id,
        icmpv6.icmpv6_type.0,
        icmpv6.icmpv6_code.0,
        icmpv6.checksum,
        icmpv6.payload.len(),
    )
}

// APPLICATION

fn bronze_dns_header(dns: &Dns, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_dns_header (
                        packet_id,
                        id,
                        is_response,
                        opcode,
                        is_authoriative,
                        is_truncated,
                        is_recursion_desirable,
                        is_recursion_available,
                        zero_reserved,
                        is_answer_authenticated,
                        is_non_authenticated_data,
                        rcode,
                        query_count,
                        response_count,
                        authority_rr_count,
                        additional_rr_count,
                        inserted_at
                    ) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, CURRENT_TIMESTAMP);"#,
        packet_id,
        dns.id,
        dns.is_response,
        dns.opcode.to_primitive_values().0,
        dns.is_authoriative,
        dns.is_truncated,
        dns.is_recursion_desirable,
        dns.is_recursion_available,
        dns.zero_reserved,
        dns.is_answer_authenticated,
        dns.is_non_authenticated_data,
        dns.rcode.to_primitive_values().0,
        dns.query_count,
        dns.response_count,
        dns.authority_rr_count,
        dns.additional_rr_count,
    )
}

fn bronze_dns_query(dns: &Dns, packet_id: u128) -> String {
    let mut request_buffer = String::new();
    if !dns.queries.is_empty() {
        request_buffer.push_str(
            r#"INSERT INTO memory.bronze_network_dns_query (
                    packet_id,
                    qname,
                    qtype,
                    qclass,
                    inserted_at
                    ) VALUES "#,
        );

        for query in &dns.queries {
            request_buffer.push_str(&format!(
                "({}, {:?}, '{}', '{}', CURRENT_TIMESTAMP),",
                packet_id, query.qname, query.qtype, query.qclass,
            ));
        }
        request_buffer.pop();
        request_buffer.push(';')
    }
    request_buffer
}

fn bronze_dns_record(dns: &Dns, packet_id: u128) -> String {
    let mut request_buffer = String::new();
    if !dns.responses.is_empty() || !dns.additional.is_empty() || !dns.authorities.is_empty() {
        request_buffer.push_str(
            r#"INSERT INTO memory.bronze_network_dns_response (
                    packet_id,
                    origin,
                    name_tag,
                    rtype,
                    rclass,
                    ttl,
                    rdlength,
                    rdata,
                    inserted_at
                    ) VALUES "#,
        );

        for response in &dns.responses {
            request_buffer.push_str(&format!(
                "({}, 0, '{}', '{}', '{}', {}, {}, {:?}, CURRENT_TIMESTAMP),",
                packet_id,
                response.name_tag,
                response.rtype,
                response.rclass,
                response.ttl,
                response.data_len,
                response.data,
            ));
        }

        for additional in &dns.additional {
            request_buffer.push_str(&format!(
                "({}, 1, '{}', '{}', '{}', {}, {}, {:?}, CURRENT_TIMESTAMP),",
                packet_id,
                additional.name_tag,
                additional.rtype,
                additional.rclass,
                additional.ttl,
                additional.data_len,
                additional.data,
            ));
        }

        for response in &dns.authorities {
            request_buffer.push_str(&format!(
                "({}, 2, '{}', '{}', '{}', {}, {}, {:?}, CURRENT_TIMESTAMP),",
                packet_id,
                response.name_tag,
                response.rtype,
                response.rclass,
                response.ttl,
                response.data_len,
                response.data,
            ));
        }
        request_buffer.pop();
        request_buffer.push(';')
    }

    request_buffer
}

fn bronze_http(http: Http, packet_id: u128) -> String {
    let method = if let Some(method) = http.instruction.method {
        format!("'{}'", method)
    } else {
        "NULL".to_string()
    };
    let uri = if let Some(uri) = http.instruction.uri {
        format!("'{}'", uri)
    } else {
        "NULL".to_string()
    };
    let status_text = if let Some(status_text) = http.instruction.status_text {
        format!("'{}'", status_text)
    } else {
        "NULL".to_string()
    };
    let status_code = if let Some(status_code) = http.instruction.status_code {
        format!("{}", status_code)
    } else {
        "NULL".to_string()
    };
    format!(
        r#"INSERT INTO memory.bronze_network_http (
                    packet_id,
                    type,
                    method,
                    uri,
                    version,
                    status_code,
                    status_text,
                    headers,
                    body,
                    inserted_at
                    ) VALUES ({}, '{}', {}, {}, '{}', {}, {}, '{}', '{}', CURRENT_TIMESTAMP);"#,
        packet_id,
        http.instruction._type,
        method,
        uri,
        http.instruction.version,
        status_code,
        status_text,
        format!("{:?}", http.headers.headers).replace('\'', "''"),
        http.body.replace('\'', "''"),
    )
}

fn bronze_tls(tls: Tls, packet_id: u128) -> String {
    format!(
        r#"INSERT INTO memory.bronze_network_tls (
                    packet_id,
                    content_type,
                    version,
                    length,
                    inserted_at
                    ) VALUES ({}, {}, {}, {}, CURRENT_TIMESTAMP);"#,
        packet_id,
        u8::from(tls.content_type),
        u16::from(tls.version),
        tls.length,
    )
}
