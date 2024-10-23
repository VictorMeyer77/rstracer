use regex::Regex;

// BRONZE

const BRONZE_PROCESS_LIST: &str = r#"
CREATE SEQUENCE IF NOT EXISTS bronze_process_list_serial;
CREATE TABLE IF NOT EXISTS bronze_process_list (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_process_list_serial'),
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
CREATE SEQUENCE IF NOT EXISTS bronze_open_files_serial;
CREATE TABLE IF NOT EXISTS bronze_open_files (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_open_files_serial'),
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
CREATE TABLE IF NOT EXISTS bronze_network_packet (
    _id UHUGEINT PRIMARY KEY,
    interface TEXT,
    length UINTEGER,
    created_at TIMESTAMP,
    inserted_at TIMESTAMP,
    brz_ingestion_duration INTERVAL
);
"#;

const BRONZE_NETWORK_INTERFACE_ADDRESS: &str = r#"
CREATE SEQUENCE IF NOT EXISTS bronze_network_interface_address_serial;
CREATE TABLE IF NOT EXISTS bronze_network_interface_address (
    _id INTEGER DEFAULT nextval('bronze_network_interface_address_serial'),
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
CREATE SEQUENCE IF NOT EXISTS bronze_network_ethernet_serial;
CREATE TABLE IF NOT EXISTS bronze_network_ethernet (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_ethernet_serial'),
    packet_id UHUGEINT,
    source TEXT,
    destination TEXT,
    ether_type USMALLINT,
    payload_length UINTEGER,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_IPV4: &str = r#"
CREATE SEQUENCE IF NOT EXISTS bronze_network_ipv4_serial;
CREATE TABLE IF NOT EXISTS bronze_network_ipv4 (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_ipv4_serial'),
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
CREATE SEQUENCE IF NOT EXISTS bronze_network_ipv6_serial;
CREATE TABLE IF NOT EXISTS bronze_network_ipv6 (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_ipv6_serial'),
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
CREATE SEQUENCE IF NOT EXISTS bronze_network_arp_serial;
CREATE TABLE IF NOT EXISTS bronze_network_arp (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_arp_serial'),
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
CREATE SEQUENCE IF NOT EXISTS bronze_network_tcp_serial;
CREATE TABLE IF NOT EXISTS bronze_network_tcp (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_tcp_serial'),
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
CREATE SEQUENCE IF NOT EXISTS bronze_network_udp_serial;
CREATE TABLE IF NOT EXISTS bronze_network_udp (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_udp_serial'),
    packet_id UHUGEINT,
    source USMALLINT,
    destination USMALLINT,
    length USMALLINT,
    checksum USMALLINT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_ICMP: &str = r#"
CREATE SEQUENCE IF NOT EXISTS bronze_network_icmp_serial;
CREATE TABLE IF NOT EXISTS bronze_network_icmp (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_icmp_serial'),
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
CREATE SEQUENCE IF NOT EXISTS bronze_network_tls_serial;
CREATE TABLE IF NOT EXISTS bronze_network_tls (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_tls_serial'),
    packet_id UHUGEINT,
    content_type USMALLINT,
    version USMALLINT,
    length USMALLINT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_DNS_HEADER: &str = r#"
CREATE SEQUENCE IF NOT EXISTS bronze_network_dns_header_serial;
CREATE TABLE IF NOT EXISTS bronze_network_dns_header (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_dns_header_serial'),
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
CREATE SEQUENCE IF NOT EXISTS bronze_network_dns_query_serial;
CREATE TABLE IF NOT EXISTS bronze_network_dns_query (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_dns_query_serial'),
    packet_id UHUGEINT,
    qname UTINYINT[],
    qtype TEXT,
    qclass TEXT,
    inserted_at TIMESTAMP,
);
"#;

const BRONZE_NETWORK_DNS_RECORD: &str = r#"
CREATE SEQUENCE IF NOT EXISTS bronze_network_dns_response_serial;
CREATE TABLE IF NOT EXISTS bronze_network_dns_response (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_dns_response_serial'),
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
CREATE SEQUENCE IF NOT EXISTS bronze_network_http_serial;
CREATE TABLE IF NOT EXISTS bronze_network_http (
    _id INTEGER PRIMARY KEY DEFAULT nextval('bronze_network_http_serial'),
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
CREATE TABLE IF NOT EXISTS silver_process_list (
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
CREATE TABLE IF NOT EXISTS silver_open_files (
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
CREATE TABLE IF NOT EXISTS silver_network_packet (
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
CREATE TABLE IF NOT EXISTS silver_network_interface_address (
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
CREATE TABLE IF NOT EXISTS silver_network_ethernet (
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
CREATE TABLE IF NOT EXISTS silver_network_dns (
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
CREATE TABLE IF NOT EXISTS silver_network_ip (
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
CREATE TABLE IF NOT EXISTS silver_network_transport (
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
CREATE TABLE IF NOT EXISTS silver_network_arp (
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
CREATE TABLE IF NOT EXISTS gold_dim_services (
    name TEXT,
    port USMALLINT,
    protocol TEXT,
    inserted_at TIMESTAMP,
    PRIMARY KEY (name, port, protocol)
);
"#;

const GOLD_DIM_HOSTS: &str = r#"
CREATE TABLE IF NOT EXISTS gold_dim_hosts (
    name TEXT,
    address TEXT,
    inserted_at TIMESTAMP,
    PRIMARY KEY (name, address)
);
"#;

// GOLD

const GOLD_PROCESS_LIST: &str = r#"
CREATE TABLE IF NOT EXISTS gold_process_list (
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
CREATE TABLE IF NOT EXISTS gold_open_files_regular (
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
CREATE TABLE IF NOT EXISTS gold_open_files_network (
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
CREATE TABLE IF NOT EXISTS gold_network_packet (
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
CREATE TABLE IF NOT EXISTS gold_network_ip (
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
CREATE TABLE IF NOT EXISTS gold_process_network (
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
CREATE TABLE IF NOT EXISTS gold_process_command (
	pid USMALLINT PRIMARY KEY,
	ppid USMALLINT,
	command TEXT,
	inserted_at TIMESTAMP
);
"#;

const GOLD_TECH_TABLE_COUNT: &str = r#"
CREATE TABLE IF NOT EXISTS gold_tech_table_count (
	_id USMALLINT PRIMARY KEY,
	name TEXT,
	min_count BIGINT,
	max_count BIGINT,
	last_count BIGINT,
	inserted_at TIMESTAMP
);
"#;

const GOLD_TECH_CHRONO: &str = r#"
CREATE TABLE IF NOT EXISTS gold_tech_chrono (
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

pub fn create_schema_request() -> String {
    format!(
        r#"{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}
           {} {} {} {} {} {} {}"#,
        BRONZE_PROCESS_LIST,
        BRONZE_OPEN_FILES,
        BRONZE_NETWORK_PACKET,
        BRONZE_NETWORK_INTERFACE_ADDRESS,
        BRONZE_NETWORK_ETHERNET,
        BRONZE_NETWORK_IPV4,
        BRONZE_NETWORK_IPV6,
        BRONZE_NETWORK_ARP,
        BRONZE_NETWORK_TCP,
        BRONZE_NETWORK_UDP,
        BRONZE_NETWORK_ICMP,
        BRONZE_NETWORK_TLS,
        BRONZE_NETWORK_DNS_HEADER,
        BRONZE_NETWORK_DNS_QUERY,
        BRONZE_NETWORK_DNS_RECORD,
        BRONZE_NETWORK_HTTP,
        SILVER_PROCESS_LIST,
        SILVER_OPEN_FILES,
        SILVER_NETWORK_PACKET,
        SILVER_NETWORK_INTERFACE_ADDRESS,
        SILVER_NETWORK_ETHERNET,
        SILVER_NETWORK_DNS,
        SILVER_NETWORK_IP,
        SILVER_NETWORK_TRANSPORT,
        SILVER_NETWORK_ARP,
        GOLD_DIM_SERVICES,
        GOLD_DIM_HOSTS,
        GOLD_PROCESS_LIST,
        GOLD_OPEN_FILES_REGULAR,
        GOLD_OPEN_FILES_NETWORK,
        GOLD_NETWORK_PACKET,
        GOLD_NETWORK_IP,
        GOLD_PROCESS_NETWORK,
        GOLD_PROCESS_COMMAND,
        GOLD_TECH_TABLE_COUNT,
        GOLD_TECH_CHRONO
    )
}

pub fn get_schema() -> Vec<String> {
    let request = create_schema_request();
    let regex = Regex::new(r"CREATE TABLE IF NOT EXISTS\s+(\w+)\s*\(").unwrap();
    let tables: Vec<String> = regex
        .captures_iter(&request)
        .map(|capture| capture.get(1).unwrap().as_str().to_string())
        .collect();
    tables
}

#[cfg(test)]
pub mod tests {
    use crate::pipeline::stage::tests::create_test_connection;

    #[test]
    fn test_create_schema() {
        let connection = create_test_connection();
        let mut statement = connection
            .prepare(
                r#"SELECT COUNT (DISTINCT table_name)
                       FROM information_schema.columns;"#,
            )
            .unwrap();
        let mut rows = statement.query([]).unwrap();

        if let Some(row) = rows.next().unwrap() {
            let count: usize = row.get(0).unwrap();
            assert_eq!(count, 36);
        }
    }
}
