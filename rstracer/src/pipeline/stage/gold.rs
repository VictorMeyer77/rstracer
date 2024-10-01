const GOLD_OPEN_FILES_NETWORK: &str = r#"
INSERT INTO memory.gold_open_files_network BY NAME
(
    SELECT
        pid,
        uid,
        ip_source_address AS source_address,
        ip_source_port AS source_port,
        ip_destination_address AS destination_address,
        ip_destination_port AS destination_port,
        inserted_at AS started_at,
        inserted_at AS updated_at,
    FROM
        (
        SELECT
            *,
            row_number() OVER (PARTITION BY pid, uid, ip_source_address, ip_source_port ORDER BY inserted_at DESC) AS row_num
        FROM
            (
                SELECT
                    pid,
                    uid,
                    ip_source_address,
                    ip_source_port,
                    ip_destination_address,
                    ip_destination_port,
                    inserted_at
                FROM
                    memory.silver_open_files
                WHERE UPPER(type) IN ('IPV4', 'IPV6')
            )
        )
    WHERE row_num = 1
)
ON CONFLICT DO UPDATE
SET updated_at = EXCLUDED.updated_at
;"#;

const GOLD_NETWORK_IP: &str = r#"
INSERT OR REPLACE INTO memory.gold_network_ip BY NAME
(
    SELECT address,
           version,
           inserted_at AS last_updated
    FROM
        (
            SELECT
            *,
            row_number() OVER (PARTITION BY address ORDER BY inserted_at DESC) AS row_num
            FROM
            (
                SELECT source AS address, version, inserted_at
                FROM memory.silver_network_ip
                UNION ALL
                SELECT destination AS address, version, inserted_at
                FROM memory.silver_network_ip
            )
        )
    WHERE row_num = 1
);
"#;

pub fn request() -> String {
    format!("{} {}", GOLD_OPEN_FILES_NETWORK, GOLD_NETWORK_IP)
}
