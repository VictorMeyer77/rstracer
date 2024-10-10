const GOLD_DIM_PROCESS: &str = r#"
INSERT OR REPLACE INTO memory.gold_fact_process BY NAME
(
    SELECT
        pid,
        ppid,
        uid,
        lstart,
        command,
        inserted_at AS updated_at
    FROM
    (
        SELECT
            pid,
            ppid,
            uid,
            lstart,
            command,
            inserted_at,
            row_number() OVER (PARTITION BY pid, lstart ORDER BY inserted_at DESC) AS row_num
        FROM memory.silver_process_list

    )
    WHERE ROW_NUM = 1
);"#;

const GOLD_OPEN_FILES_REGULAR: &str = r#"
INSERT INTO memory.gold_open_files_regular BY NAME
(
    SELECT
        pid,
        uid,
        name,
        fd,
        node,
        command,
        _id AS silver_id,
        created_at AS started_at,
        created_at AS updated_at
    FROM
        (
            SELECT
                *,
                ROW_NUMBER() OVER (PARTITION BY pid, fd, node ORDER BY created_at DESC) AS row_num
            FROM
                memory.silver_open_files
            WHERE UPPER(type) NOT IN ('IPV4', 'IPV6')
        )
    WHERE row_num = 1
)
ON CONFLICT DO UPDATE
SET updated_at = EXCLUDED.updated_at
;"#;

const GOLD_OPEN_FILES_NETWORK: &str = r#"
INSERT INTO memory.gold_open_files_network BY NAME
(
    SELECT DISTINCT
        HASH(pid, fd, source_address, source_port, destination_address, destination_port) AS _id,
        pid,
        uid,
        command,
        source_address::INET AS source_address,
        source_port,
        destination_address,
        destination_port,
        silver_id,
        created_at AS started_at,
        created_at AS updated_at
    FROM
    (
        SELECT
            ofn.pid,
            ofn.uid,
            ofn.fd,
            ofn.command,
            CASE
                WHEN ofn.ip_source_address = '*' THEN NULL
                ELSE REPLACE(COALESCE(hos1.address, ofn.ip_source_address), '-', ':')
            END AS source_address,
            CASE
                WHEN ofn.ip_source_port IN ('*', '') THEN NULL
                ELSE COALESCE(ser1.port::TEXT, ofn.ip_source_port)
            END AS source_port,
            CASE
                WHEN ofn.ip_destination_address = '*' THEN NULL
                ELSE COALESCE(hos2.address, ofn.ip_destination_address)
            END AS destination_address,
            CASE
                WHEN ofn.ip_destination_port IN ('*', '') THEN NULL
                ELSE COALESCE(ser2.port::TEXT, ofn.ip_destination_port)
            END AS destination_port,
            ofn.created_at,
            ofn._id AS silver_id
        FROM
        (
            SELECT
                *,
                ROW_NUMBER() OVER (PARTITION BY pid, fd, ip_source_address, ip_source_port, ip_destination_address, ip_destination_port ORDER BY created_at DESC) AS row_num
            FROM memory.silver_open_files
            WHERE UPPER(type) IN ('IPV4', 'IPV6')
        ) ofn
        LEFT JOIN memory.gold_dim_hosts hos1 ON LOWER(ofn.ip_source_address) = LOWER(hos1.name)
        LEFT JOIN memory.gold_dim_hosts hos2 ON LOWER(ofn.ip_destination_address) = LOWER(hos2.name)
        LEFT JOIN memory.gold_dim_services ser1 ON LOWER(ofn.ip_source_port) = LOWER(ser1.name)
        LEFT JOIN memory.gold_dim_services ser2 ON LOWER(ofn.ip_destination_port) = LOWER(ser2.name)
        WHERE ofn.row_num = 1
    )
)
ON CONFLICT DO UPDATE
SET updated_at = EXCLUDED.updated_at
;"#;

const GOLD_NETWORK_FACT_IP: &str = r#"
INSERT OR REPLACE INTO memory.gold_network_fact_ip BY NAME
(
    SELECT
        packet._id AS _id,
        ip.version AS ip_version,
        transport.protocol AS transport_protocol,
        ip.source AS source_address,
        transport.source AS source_port,
        ip.destination AS destination_address,
        transport.destination AS destination_port,
        packet.created_at,
        packet.inserted_at
    FROM memory.silver_network_transport transport
    INNER JOIN memory.silver_network_ip ip ON transport._id = ip._id
    INNER JOIN memory.silver_network_packet packet ON packet._id = ip._id
);
"#;

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
    format!(
        "{} {} {} {} {}",
        GOLD_DIM_PROCESS,
        GOLD_OPEN_FILES_REGULAR,
        GOLD_OPEN_FILES_NETWORK,
        GOLD_NETWORK_IP,
        GOLD_NETWORK_FACT_IP
    )
}
