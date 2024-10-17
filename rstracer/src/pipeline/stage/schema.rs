const CREATE_DATABASE: &str = r#"
ATTACH '{disk_file_path}' AS disk;
"#;

pub const GET_SCHEMA: &str = r#"
    SELECT table_name,
    string_agg(column_name, ';'),
    string_agg(data_type, ';'),
    string_agg(is_nullable, ';'),
    string_agg((column_default IS NOT NULL AND starts_with(column_default, 'nextval')), ';')
    FROM information_schema.columns
    WHERE table_catalog = 'memory'
    GROUP BY table_name;"#;

// BRONZE

const BRONZE_PROCESS_LIST: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_process_list_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_process_list (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_process_list_serial'),
    pid INTEGER,
    ppid INTEGER,
    uid INTEGER,
    lstart TIMESTAMP,
    pcpu FLOAT,
    pmem FLOAT,
    status TEXT,
    command TEXT,
    created_at TIMESTAMP,
    inserted_at TIMESTAMP,
    brz_ingestion_duration INTERVAL
);
"#;

const BRONZE_OPEN_FILES: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_open_files_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_open_files (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_open_files_serial'),
    command TEXT,
    pid INTEGER,
    uid INTEGER,
    fd TEXT,
    type TEXT,
    device TEXT,
    size BIGINT,
    node TEXT,
    name TEXT,
    created_at TIMESTAMP,
    inserted_at TIMESTAMP,
    brz_ingestion_duration INTERVAL
);
"#;

const BRONZE_NETWORK_PACKET: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_packet (
    _id UHUGEINT PRIMARY KEY,
    interface TEXT,
    length UINTEGER,
    created_at TIMESTAMP,
    inserted_at TIMESTAMP,
    brz_ingestion_duration INTERVAL
);
"#;

const BRONZE_NETWORK_INTERFACE_ADDRESS: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_interface_address_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_interface_address (
    _id INTEGER DEFAULT nextval('{db_name}.bronze_network_interface_address_serial'),
    interface TEXT,
    address TEXT,
    netmask TEXT,
    broadcast_address TEXT,
    destination_address TEXT,
    inserted_at TIMESTAMP,
    PRIMARY KEY (interface, address)
);
"#;

const BRONZE_NETWORK_ETHERNET: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_ethernet_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_ethernet (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_ethernet_serial'),
    packet_id UHUGEINT,
    source TEXT,
    destination TEXT,
    ether_type USMALLINT,
    payload_length UINTEGER,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_IPV4: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_ipv4_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_ipv4 (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_ipv4_serial'),
    packet_id UHUGEINT,
    version USMALLINT,
    header_length USMALLINT,
    dscp USMALLINT,
    ecn USMALLINT,
    total_length USMALLINT,
    identification USMALLINT,
    flags USMALLINT,
    fragment_offset USMALLINT,
    ttl USMALLINT,
    next_level_protocol USMALLINT,
    checksum USMALLINT,
    source TEXT,
    destination TEXT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_IPV6: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_ipv6_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_ipv6 (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_ipv6_serial'),
    packet_id UHUGEINT,
    version USMALLINT,
    traffic_class USMALLINT,
    flow_label UINTEGER,
    payload_length USMALLINT,
    next_header USMALLINT,
    hop_limit USMALLINT,
    source TEXT,
    destination TEXT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_ARP: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_arp_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_arp (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_arp_serial'),
    packet_id UHUGEINT,
    hardware_type USMALLINT,
    protocol_type USMALLINT,
    hw_addr_len USMALLINT,
    proto_addr_len USMALLINT,
    operation USMALLINT,
    sender_hw_addr TEXT,
    sender_proto_addr TEXT,
    target_hw_addr TEXT,
    target_proto_addr TEXT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_TCP: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_tcp_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_tcp (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_tcp_serial'),
    packet_id UHUGEINT,
    source USMALLINT,
    destination USMALLINT,
    sequence UINTEGER,
    acknowledgement UINTEGER,
    data_offset USMALLINT,
    reserved USMALLINT,
    flags USMALLINT,
    _window USMALLINT,
    checksum USMALLINT,
    urgent_ptr USMALLINT,
    options TEXT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_UDP: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_udp_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_udp (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_udp_serial'),
    packet_id UHUGEINT,
    source USMALLINT,
    destination USMALLINT,
    length USMALLINT,
    checksum USMALLINT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_ICMP: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_icmp_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_icmp (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_icmp_serial'),
    packet_id UHUGEINT,
    version USMALLINT,
    type USMALLINT,
    code USMALLINT,
    checksum USMALLINT,
    payload_length UINTEGER,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_TLS: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_tls_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_tls (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_tls_serial'),
    packet_id UHUGEINT,
    content_type USMALLINT,
    version USMALLINT,
    length USMALLINT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_DNS_HEADER: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_dns_header_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_dns_header (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_dns_header_serial'),
    packet_id UHUGEINT,
    id USMALLINT,
    is_response USMALLINT,
    opcode USMALLINT,
    is_authoriative USMALLINT,
    is_truncated USMALLINT,
    is_recursion_desirable USMALLINT,
    is_recursion_available USMALLINT,
    zero_reserved USMALLINT,
    is_answer_authenticated USMALLINT,
    is_non_authenticated_data USMALLINT,
    rcode USMALLINT,
    query_count USMALLINT,
    response_count USMALLINT,
    authority_rr_count USMALLINT,
    additional_rr_count USMALLINT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_DNS_QUERY: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_dns_query_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_dns_query (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_dns_query_serial'),
    packet_id UHUGEINT,
    qname UTINYINT[],
    qtype TEXT,
    qclass TEXT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_DNS_RECORD: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_dns_response_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_dns_response (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_dns_response_serial'),
    packet_id UHUGEINT,
    origin USMALLINT,
    name_tag USMALLINT,
    rtype TEXT,
    rclass TEXT,
    ttl UINTEGER,
    rdlength USMALLINT,
    rdata UTINYINT[],
    inserted_at TIMESTAMP,
);"#;

const BRONZE_NETWORK_HTTP: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.bronze_network_http_serial;
CREATE TABLE IF NOT EXISTS {db_name}.bronze_network_http (
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_network_http_serial'),
    packet_id UHUGEINT,
    type TEXT,
    method TEXT,
    uri TEXT,
    version TEXT,
    status_code USMALLINT,
    status_text TEXT,
    headers TEXT,
    body TEXT,
    inserted_at TIMESTAMP,
);
"#;

// SILVER

const SILVER_PROCESS_LIST: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.silver_process_list (
    _id INTEGER PRIMARY KEY,
    pid INTEGER,
    ppid INTEGER,
    uid INTEGER,
    lstart TIMESTAMP,
    pcpu FLOAT,
    pmem FLOAT,
    status TEXT,
    command TEXT,
    created_at TIMESTAMP,
    brz_ingestion_duration INTERVAL,
    duration INTERVAL,
    inserted_at TIMESTAMP,
    svr_ingestion_duration INTERVAL
);
"#;

const SILVER_OPEN_FILES: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.silver_open_files (
    _id INTEGER PRIMARY KEY,
    command TEXT,
    pid INTEGER,
    uid INTEGER,
    fd TEXT,
    type TEXT,
    device TEXT,
    size BIGINT,
    node TEXT,
    name TEXT,
    created_at TIMESTAMP,
    brz_ingestion_duration INTERVAL,
    ip_source_address TEXT,
    ip_source_port TEXT,
    ip_destination_address TEXT,
    ip_destination_port TEXT,
    inserted_at TIMESTAMP,
    svr_ingestion_duration INTERVAL
);
"#;

const SILVER_NETWORK_PACKET: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.silver_network_packet (
    _id UHUGEINT PRIMARY KEY,
    interface TEXT,
    length UINTEGER,
    created_at TIMESTAMP,
    brz_ingestion_duration INTERVAL,
    data_link TEXT,
    network TEXT,
    transport TEXT,
    application TEXT,
    inserted_at TIMESTAMP,
    svr_ingestion_duration INTERVAL
);
"#;

const SILVER_NETWORK_INTERFACE_ADDRESS: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.silver_network_interface_address (
    _id INTEGER PRIMARY KEY,
    interface TEXT,
    address INET,
    broadcast_address INET,
    destination_address INET,
    inserted_at TIMESTAMP,
    svr_ingestion_duration INTERVAL,
);
"#;

const SILVER_NETWORK_ETHERNET: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.silver_network_ethernet (
    _id UHUGEINT PRIMARY KEY,
    source TEXT,
    destination TEXT,
    ether_type USMALLINT,
    payload_length UINTEGER,
    packet_length UINTEGER,
    interface TEXT,
    created_at TIMESTAMP,
    brz_ingestion_duration INTERVAL,
    inserted_at TIMESTAMP,
    svr_ingestion_duration INTERVAL
);
"#;

const SILVER_NETWORK_DNS: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.silver_network_dns (
    _id TEXT PRIMARY KEY,
    packet_id UHUGEINT,
    id USMALLINT,
    is_response USMALLINT,
    opcode USMALLINT,
    is_authoriative USMALLINT,
    is_truncated USMALLINT,
    is_recursion_desirable USMALLINT,
    is_recursion_available USMALLINT,
    zero_reserved USMALLINT,
    is_answer_authenticated USMALLINT,
    is_non_authenticated_data USMALLINT,
    rcode USMALLINT,
    query_count USMALLINT,
    response_count USMALLINT,
    authority_rr_count USMALLINT,
    additional_rr_count USMALLINT,
    qname UTINYINT[],
    qtype TEXT,
    qclass TEXT,
    origin USMALLINT,
    name_tag USMALLINT,
    rtype TEXT,
    rclass TEXT,
    ttl UINTEGER,
    rdlength USMALLINT,
    rdata UTINYINT[],
    created_at TIMESTAMP,
    brz_ingestion_duration INTERVAL,
    question_parsed TEXT,
    response_parsed TEXT,
    inserted_at TIMESTAMP,
    svr_ingestion_duration INTERVAL
);
"#;

const SILVER_NETWORK_IP: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.silver_network_ip (
    _id UHUGEINT PRIMARY KEY,
    version USMALLINT,
    length USMALLINT,
    hop_limit USMALLINT,
    next_protocol USMALLINT,
    source INET,
    destination INET,
    packet_length UINTEGER,
    interface TEXT,
    created_at TIMESTAMP,
    brz_ingestion_duration INTERVAL,
    inserted_at TIMESTAMP,
    svr_ingestion_duration INTERVAL
);
"#;

const SILVER_NETWORK_TRANSPORT: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.silver_network_transport (
    _id UHUGEINT PRIMARY KEY,
    protocol TEXT,
    source USMALLINT,
    destination USMALLINT,
    packet_length UINTEGER,
    interface TEXT,
    created_at TIMESTAMP,
    brz_ingestion_duration INTERVAL,
    inserted_at TIMESTAMP,
    svr_ingestion_duration INTERVAL
);
"#;

const SILVER_NETWORK_ARP: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.silver_network_arp (
    _id UHUGEINT PRIMARY KEY,
    hardware_type USMALLINT,
    protocol_type USMALLINT,
    hw_addr_len USMALLINT,
    proto_addr_len USMALLINT,
    operation USMALLINT,
    sender_hw_addr TEXT,
    sender_proto_addr TEXT,
    target_hw_addr TEXT,
    target_proto_addr TEXT,
    packet_length UINTEGER,
    interface TEXT,
    created_at TIMESTAMP,
    brz_ingestion_duration INTERVAL,
    inserted_at TIMESTAMP,
    svr_ingestion_duration INTERVAL
);
"#;

// DIM

const GOLD_DIM_SERVICES: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_dim_services (
    name TEXT,
    port USMALLINT,
    protocol TEXT,
    inserted_at TIMESTAMP,
    PRIMARY KEY (name, port, protocol)
);
"#;

const GOLD_DIM_HOSTS: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_dim_hosts (
    name TEXT,
    address TEXT,
    inserted_at TIMESTAMP,
    PRIMARY KEY (name, address)
);
"#;

// GOLD

const GOLD_PROCESS_LIST: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_process_list (
    pid USMALLINT,
    ppid USMALLINT,
    uid USMALLINT,
    command TEXT,
    min_pcpu FLOAT,
    max_pcpu FLOAT,
    last_pcpu FLOAT,
    min_pmem FLOAT,
    max_pmem FLOAT,
    last_pmem FLOAT,
    silver_id BIGINT,
    started_at TIMESTAMP,
    inserted_at TIMESTAMP,
    PRIMARY KEY (pid, started_at)
);
"#;

const GOLD_OPEN_FILES_REGULAR: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_open_files_regular (
    pid USMALLINT,
    uid USMALLINT,
    fd TEXT,
    node TEXT,
    command TEXT,
    name TEXT,
    min_size BIGINT,
    max_size BIGINT,
    last_size BIGINT,
    silver_id INTEGER,
    started_at TIMESTAMP,
    inserted_at TIMESTAMP,
    PRIMARY KEY (pid, fd, node)
);
"#;

const GOLD_OPEN_FILES_NETWORK: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_open_files_network (
    _id UBIGINT PRIMARY KEY,
    pid USMALLINT,
    uid USMALLINT,
    command TEXT,
    source_address INET,
    source_port USMALLINT,
    destination_address TEXT,
    destination_port USMALLINT,
    silver_id INTEGER,
    started_at TIMESTAMP,
    inserted_at TIMESTAMP
);
"#;

const GOLD_NETWORK_PACKET: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_network_packet (
    _id UHUGEINT PRIMARY KEY,
    interface TEXT,
    length UINTEGER,
    created_at TIMESTAMP,
    data_link TEXT,
    network TEXT,
    transport TEXT,
    application TEXT,
    inserted_at TIMESTAMP
);
"#;

const GOLD_NETWORK_IP: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_network_ip (
    _id UHUGEINT PRIMARY KEY,
    ip_version UTINYINT,
    transport_protocol TEXT,
    source_address INET,
    source_port TEXT,
    destination_address INET,
    destination_port TEXT,
    created_at TIMESTAMP,
    inserted_at TIMESTAMP
);
"#;

const GOLD_PROCESS_NETWORK: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_process_network (
    _id UBIGINT PRIMARY KEY,
	pid USMALLINT,
	uid USMALLINT,
	command TEXT,
	source_address INET,
	source_port USMALLINT,
	destination_address INET,
	destination_port USMALLINT,
	is_source BOOL,
	process_svr_id INTEGER,
	open_file_svr_id INTEGER,
	packet_id UHUGEINT,
    inserted_at TIMESTAMP
);
"#;

const GOLD_PROCESS_COMMAND: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_process_command (
	pid USMALLINT PRIMARY KEY,
	ppid USMALLINT,
	command TEXT,
	inserted_at TIMESTAMP
);
"#;

const GOLD_TECH_TABLE_COUNT: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_tech_table_count (
	_id USMALLINT PRIMARY KEY,
	name TEXT,
	min_count BIGINT,
	max_count BIGINT,
	last_count BIGINT,
	inserted_at TIMESTAMP
);
"#;

const GOLD_TECH_CHRONO: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_tech_chrono (
	name TEXT PRIMARY KEY,
	brz_max_ingest FLOAT,
	brz_min_ingest FLOAT,
	svr_max_ingest FLOAT,
	svr_min_ingest FLOAT,
	max_ingest FLOAT,
	min_ingest FLOAT,
	inserted_at TIMESTAMP
);
"#;

#[derive(Debug, Clone)]
pub struct Schema {
    pub tables: Vec<Table>,
}
#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub type_name: String,
    pub is_nullable: bool,
    pub is_autoincrement: bool,
}

fn create_tables_request(database: &str) -> String {
    format!(
        r#"{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}
           {} {} {} {} {} {} {}"#,
        BRONZE_PROCESS_LIST.replace("{db_name}", database),
        BRONZE_OPEN_FILES.replace("{db_name}", database),
        BRONZE_NETWORK_PACKET.replace("{db_name}", database),
        BRONZE_NETWORK_INTERFACE_ADDRESS.replace("{db_name}", database),
        BRONZE_NETWORK_ETHERNET.replace("{db_name}", database),
        BRONZE_NETWORK_IPV4.replace("{db_name}", database),
        BRONZE_NETWORK_IPV6.replace("{db_name}", database),
        BRONZE_NETWORK_ARP.replace("{db_name}", database),
        BRONZE_NETWORK_TCP.replace("{db_name}", database),
        BRONZE_NETWORK_UDP.replace("{db_name}", database),
        BRONZE_NETWORK_ICMP.replace("{db_name}", database),
        BRONZE_NETWORK_TLS.replace("{db_name}", database),
        BRONZE_NETWORK_DNS_HEADER.replace("{db_name}", database),
        BRONZE_NETWORK_DNS_QUERY.replace("{db_name}", database),
        BRONZE_NETWORK_DNS_RECORD.replace("{db_name}", database),
        BRONZE_NETWORK_HTTP.replace("{db_name}", database),
        SILVER_PROCESS_LIST.replace("{db_name}", database),
        SILVER_OPEN_FILES.replace("{db_name}", database),
        SILVER_NETWORK_PACKET.replace("{db_name}", database),
        SILVER_NETWORK_INTERFACE_ADDRESS.replace("{db_name}", database),
        SILVER_NETWORK_ETHERNET.replace("{db_name}", database),
        SILVER_NETWORK_DNS.replace("{db_name}", database),
        SILVER_NETWORK_IP.replace("{db_name}", database),
        SILVER_NETWORK_TRANSPORT.replace("{db_name}", database),
        SILVER_NETWORK_ARP.replace("{db_name}", database),
        GOLD_DIM_SERVICES.replace("{db_name}", database),
        GOLD_DIM_HOSTS.replace("{db_name}", database),
        GOLD_PROCESS_LIST.replace("{db_name}", database),
        GOLD_OPEN_FILES_REGULAR.replace("{db_name}", database),
        GOLD_OPEN_FILES_NETWORK.replace("{db_name}", database),
        GOLD_NETWORK_PACKET.replace("{db_name}", database),
        GOLD_NETWORK_IP.replace("{db_name}", database),
        GOLD_PROCESS_NETWORK.replace("{db_name}", database),
        GOLD_PROCESS_COMMAND.replace("{db_name}", database),
        GOLD_TECH_TABLE_COUNT.replace("{db_name}", database),
        GOLD_TECH_CHRONO.replace("{db_name}", database)
    )
}

pub fn create_schema_request(disk_file_path: &str) -> String {
    format!(
        "{} {} {}",
        CREATE_DATABASE.replace("{disk_file_path}", disk_file_path),
        create_tables_request("memory"),
        create_tables_request("disk")
    )
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::pipeline::stage::tests::get_test_connection;

    pub fn create_mock_schema() -> Schema {
        Schema {
            tables: vec![
                Table {
                    name: "bronze_process_list".to_string(),
                    columns: vec![
                        Column {
                            name: "pid".to_string(),
                            type_name: "USMALLINT".to_string(),
                            is_nullable: false,
                            is_autoincrement: false,
                        },
                        Column {
                            name: "inserted_at".to_string(),
                            type_name: "TIMESTAMP".to_string(),
                            is_nullable: true,
                            is_autoincrement: false,
                        },
                    ],
                },
                Table {
                    name: "silver_process_list".to_string(),
                    columns: vec![
                        Column {
                            name: "pid".to_string(),
                            type_name: "USMALLINT".to_string(),
                            is_nullable: false,
                            is_autoincrement: false,
                        },
                        Column {
                            name: "inserted_at".to_string(),
                            type_name: "TIMESTAMP".to_string(),
                            is_nullable: true,
                            is_autoincrement: false,
                        },
                    ],
                },
                Table {
                    name: "gold_process_list".to_string(),
                    columns: vec![
                        Column {
                            name: "pid".to_string(),
                            type_name: "USMALLINT".to_string(),
                            is_nullable: false,
                            is_autoincrement: false,
                        },
                        Column {
                            name: "inserted_at".to_string(),
                            type_name: "TIMESTAMP".to_string(),
                            is_nullable: true,
                            is_autoincrement: false,
                        },
                    ],
                },
                Table {
                    name: "dim_hosts".to_string(),
                    columns: vec![
                        Column {
                            name: "name".to_string(),
                            type_name: "TEXT".to_string(),
                            is_nullable: false,
                            is_autoincrement: false,
                        },
                        Column {
                            name: "inserted_at".to_string(),
                            type_name: "TIMESTAMP".to_string(),
                            is_nullable: true,
                            is_autoincrement: false,
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_create_database() {
        let connection = get_test_connection();
        let mut statement = connection
            .prepare(
                r#"
                        SELECT table_name, table_catalog
                        FROM information_schema.columns
                        GROUP BY table_name, table_catalog;"#,
            )
            .unwrap();
        let mut rows = statement.query([]).unwrap();
        let mut disk_count = 0;
        let mut mem_count = 0;

        while let Some(row) = rows.next().unwrap() {
            let table_catalog: String = row.get(1).unwrap();
            if table_catalog == "disk" {
                disk_count += 1;
            } else if table_catalog == "memory" {
                mem_count += 1;
            }
        }

        assert_eq!(disk_count, 36);
        assert_eq!(mem_count, 36);
    }
}
