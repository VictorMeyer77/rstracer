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
    _id INTEGER PRIMARY KEY DEFAULT nextval('{db_name}.bronze_process_list_serial'),    -- Primary Key
    pid INTEGER,                                                             -- Foreign Key for Process ID
    ppid INTEGER,                                                            -- Foreign Key for Parent Process ID
    uid INTEGER,                                                             -- Foreign Key for User ID
    lstart TIMESTAMP,                                                        -- Timestamp for process start time
    pcpu FLOAT,                                                              -- Percentage of CPU usage
    pmem FLOAT,                                                              -- Percentage of Memory usage
    status TEXT,                                                          -- Status of the process
    command TEXT,                                                         -- Command that initiated the process
    created_at TIMESTAMP,                                                    -- Timestamp of evenement
    inserted_at TIMESTAMP,                                                    -- Timestamp of record insertion
    brz_ingestion_duration INTERVAL                                       -- Duration between the creation and the insertion in bronze
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
    _id INTEGER PRIMARY KEY,                                                    -- Primary Key
    pid INTEGER,                                                             -- Foreign Key for Process ID
    ppid INTEGER,                                                            -- Foreign Key for Parent Process ID
    uid INTEGER,                                                             -- Foreign Key for User ID
    lstart TIMESTAMP,                                                        -- Timestamp for process start time
    pcpu FLOAT,                                                              -- Percentage of CPU usage
    pmem FLOAT,                                                              -- Percentage of Memory usage
    status TEXT,                                                          -- Status of the process
    command TEXT,                                                         -- Command that initiated the process
    created_at TIMESTAMP,                                                    -- Timestamp of evenement
    brz_ingestion_duration INTERVAL,
    duration INTERVAL,                                                -- Duration of the process
    inserted_at TIMESTAMP,                                              -- Timestamp of record insertion
    svr_ingestion_duration INTERVAL                                     -- Duration between the insertion in bronze and insertion in silver
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
    source TEXT,
    destination TEXT,
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

// GOLD

const GOLD_OPEN_FILES_REGULAR: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.gold_open_files_regular_serial;
CREATE TABLE IF NOT EXISTS {db_name}.gold_open_files_regular (
    _id INTEGER DEFAULT nextval('{db_name}.gold_open_files_regular_serial'),
    pid USMALLINT,
    uid USMALLINT,
    name TEXT,
    fd TEXT,
    node TEXT,
    min_size BIGINT,
    max_size BIGINT,
    started_at TIMESTAMP,
    updated_at TIMESTAMP,
    PRIMARY KEY (pid, uid, name, fd, node)
);
"#;

const GOLD_OPEN_FILES_NETWORK: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.gold_open_files_network_serial;
CREATE TABLE IF NOT EXISTS {db_name}.gold_open_files_network (
    _id INTEGER DEFAULT nextval('{db_name}.gold_open_files_network_serial'),
    pid USMALLINT,
    uid USMALLINT,
    fd TEXT,
    source_address TEXT,
    source_port TEXT,
    destination_address TEXT,
    destination_port TEXT,
    started_at TIMESTAMP,
    updated_at TIMESTAMP,
    PRIMARY KEY (pid, uid, fd, source_address, source_port)
);
"#;

const GOLD_NETWORK_FACT_IP: &str = r#"
CREATE TABLE IF NOT EXISTS {db_name}.gold_network_fact_ip (
    _id UHUGEINT PRIMARY KEY,
    ip_version UTINYINT,
    transport_protocol TEXT,
    source_address TEXT,
    source_port TEXT,
    destination_address TEXT,
    destination_port TEXT,
    created_at TIMESTAMP,
    inserted_at TIMESTAMP
);
"#;

const GOLD_NETWORK_IP: &str = r#"
CREATE SEQUENCE IF NOT EXISTS {db_name}.gold_network_ip_serial;
CREATE TABLE IF NOT EXISTS {db_name}.gold_network_ip (
    _id INTEGER DEFAULT nextval('{db_name}.gold_network_ip_serial'),
    address TEXT PRIMARY KEY,
    version UTINYINT,
    last_updated TIMESTAMP
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
        "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        BRONZE_PROCESS_LIST.replace("{db_name}", database),
        BRONZE_OPEN_FILES.replace("{db_name}", database),
        BRONZE_NETWORK_PACKET.replace("{db_name}", database),
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
        SILVER_NETWORK_ETHERNET.replace("{db_name}", database),
        SILVER_NETWORK_DNS.replace("{db_name}", database),
        SILVER_NETWORK_IP.replace("{db_name}", database),
        SILVER_NETWORK_TRANSPORT.replace("{db_name}", database),
        SILVER_NETWORK_ARP.replace("{db_name}", database),
        GOLD_OPEN_FILES_REGULAR.replace("{db_name}", database),
        GOLD_OPEN_FILES_NETWORK.replace("{db_name}", database),
        GOLD_NETWORK_FACT_IP.replace("{db_name}", database),
        GOLD_NETWORK_IP.replace("{db_name}", database)
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
