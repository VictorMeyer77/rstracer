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
    inserted_at TIMESTAMP                                                    -- Timestamp of record insertion
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
    duration INTERVAL,                                                -- Duration of the process
    ingestion_duration INTERVAL,                                       -- Duration between the command execution and the insertion in the database
    inserted_at TIMESTAMP                                                    -- Timestamp of record insertion
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
        "{} {}",
        BRONZE_PROCESS_LIST.replace("{db_name}", database),
        SILVER_PROCESS_LIST.replace("{db_name}", database)
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
