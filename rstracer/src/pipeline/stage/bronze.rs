use lsof::lsof::OpenFile;
use network::capture::data_link::{DataLink, DataLinkProtocol};
use network::capture::network::{Network, NetworkProtocol};
use network::capture::Capture;
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
        let row_id = Uuid::new_v4().as_u128();
        let mut request_buffer = format!(
            r#"INSERT OR REPLACE INTO memory.bronze_network_packet
        (_id, interface, length, created_at, inserted_at, brz_ingestion_duration) VALUES
        ({}, '{}', {}, TO_TIMESTAMP({3}), CURRENT_TIMESTAMP, AGE(TO_TIMESTAMP({3})::TIMESTAMP));"#,
            row_id,
            self.device.name,
            self.packet.len(),
            self.created_at
        );

        let clone = self.clone();

        if clone.data_link.is_some() {
            request_buffer.push_str(&clone.data_link.unwrap().to_insert_sql(Some(row_id)))
        }

        if clone.network.is_some() {
            request_buffer.push_str(&clone.network.unwrap().to_insert_sql(Some(row_id)))
        }

        request_buffer
    }
}

impl Bronze for DataLink {
    fn to_insert_sql(&self, foreign_id: Option<u128>) -> String {
        match self.protocol {
            DataLinkProtocol::Ethernet => {
                let ethernet = self.ethernet.clone().unwrap();
                format!(
                    r#"INSERT INTO memory.bronze_network_ethernet
        (packet_id, source, destination, ether_type, payload_length, inserted_at) VALUES
        ({}, '{}', '{}', {}, {}, CURRENT_TIMESTAMP);"#,
                    foreign_id.unwrap(),
                    ethernet.source,
                    ethernet.destination,
                    ethernet.ethertype.0,
                    ethernet.payload.len()
                )
            }
        }
    }
}

impl Bronze for Network {
    fn to_insert_sql(&self, foreign_id: Option<u128>) -> String {
        match self.protocol {
            NetworkProtocol::Arp => {
                let arp = self.arp.clone().unwrap();
                format!(
                    r#"INSERT INTO memory.bronze_network_arp (
    packet_id, hardware_type, protocol_type, hw_addr_len, proto_addr_len, operation, sender_hw_addr,
    sender_proto_addr, target_hw_addr, target_proto_addr, inserted_at
) VALUES ({}, {}, {}, {}, {}, {}, '{}', '{}', '{}', '{}', CURRENT_TIMESTAMP);"#,
                    foreign_id.unwrap(),
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
            NetworkProtocol::Ipv4 => {
                let ipv4 = self.ipv4.clone().unwrap();
                format!(
                    r#"INSERT INTO memory.bronze_network_ipv4 (
    packet_id, version, header_length, dscp, ecn, total_length,
    identification, flags, fragment_offset, ttl, next_level_protocol,
    checksum, source, destination, inserted_at
) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, '{}', '{}', CURRENT_TIMESTAMP);"#,
                    foreign_id.unwrap(),
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
            NetworkProtocol::Ipv6 => {
                let ipv6 = self.ipv6.clone().unwrap();
                format!(
                    r#"INSERT INTO memory.bronze_network_ipv6 (
    packet_id, version, traffic_class, flow_label, payload_length, next_header,
    hop_limit, source, destination, inserted_at
) VALUES ({}, {}, {}, {}, {}, {}, {}, '{}', '{}', CURRENT_TIMESTAMP);"#,
                    foreign_id.unwrap(),
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
        }
    }
}
