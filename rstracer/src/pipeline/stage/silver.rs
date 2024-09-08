use chrono::Local;

const SILVER_INGEST_PROCESS_LIST: &str = r#"
INSERT OR IGNORE INTO memory.silver_process_list BY NAME
(
SELECT
    brz._id,
    brz.pid,
    brz.ppid,
    brz.uid,
    brz.lstart - INTERVAL '{local_minus_utc} seconds' AS lstart,
    brz.pcpu,
    brz.pmem,
    brz.status,
    brz.command,
    brz.created_at,
    age(brz.created_at, brz.lstart - INTERVAL '{local_minus_utc} seconds') AS duration,
    age(brz.inserted_at, brz.created_at) AS ingestion_duration,
    current_timestamp AS inserted_at
FROM bronze_process_list brz
);
"#;

pub fn silver_request() -> String {
    let offset_in_sec = Local::now().offset().local_minus_utc().to_string();
    SILVER_INGEST_PROCESS_LIST.replace("{local_minus_utc}", &offset_in_sec)
}
