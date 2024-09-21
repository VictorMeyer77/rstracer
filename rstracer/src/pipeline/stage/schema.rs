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
    length INTEGER,
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
        "{} {} {} {} {} {} {} {} {}",
        BRONZE_PROCESS_LIST.replace("{db_name}", database),
        BRONZE_OPEN_FILES.replace("{db_name}", database),
        BRONZE_NETWORK_PACKET.replace("{db_name}", database),
        BRONZE_NETWORK_ETHERNET.replace("{db_name}", database),
        BRONZE_NETWORK_IPV4.replace("{db_name}", database),
        BRONZE_NETWORK_IPV6.replace("{db_name}", database),
        BRONZE_NETWORK_ARP.replace("{db_name}", database),
        SILVER_PROCESS_LIST.replace("{db_name}", database),
        SILVER_OPEN_FILES.replace("{db_name}", database),
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
