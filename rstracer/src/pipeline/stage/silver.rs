const SILVER_PROCESS_LIST: &str = r#"
INSERT OR IGNORE INTO memory.silver_process_list BY NAME
(
SELECT
    _id,
    pid,
    ppid,
    uid,
    lstart,
    pcpu,
    pmem,
    status,
    command,
    created_at,
    brz_ingestion_duration,
    AGE(created_at, lstart) AS duration,
    CURRENT_TIMESTAMP AS inserted_at,
    AGE(inserted_at) AS svr_ingestion_duration
FROM bronze_process_list
);
"#;

const SILVER_OPEN_FILES: &str = r#"
INSERT OR IGNORE INTO memory.silver_open_files BY NAME
(
SELECT
    _id,
    command,
    pid,
    uid,
    fd,
    type,
    device,
    size,
    node,
    name,
    created_at,
    brz_ingestion_duration,

    CASE WHEN UPPER(type) IN ('IPV4', 'IPV6') THEN SPLIT(name, ':')[1]
    ELSE NULL
    END AS ip_source_address,

    CASE WHEN UPPER(type) IN ('IPV4', 'IPV6') THEN SPLIT(split(name, ':')[2], '->')[1]
    ELSE NULL
    END AS ip_source_port,

    CASE WHEN UPPER(type) IN ('IPV4', 'IPV6') THEN SPLIT(split(name, ':')[2], '->')[2]
    ELSE NULL
    END AS ip_destination_address,

    CASE WHEN UPPER(type) IN ('IPV4', 'IPV6') THEN SPLIT(name, ':')[3]
    ELSE NULL
    END AS ip_destination_port,

    CURRENT_TIMESTAMP AS inserted_at,
    AGE(inserted_at) AS svr_ingestion_duration
FROM bronze_open_files
);
"#;

pub fn silver_request() -> String {
    format!("{} {}", SILVER_PROCESS_LIST, SILVER_OPEN_FILES)
}
