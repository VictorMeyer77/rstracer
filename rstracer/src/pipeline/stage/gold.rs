const GOLD_DIM_PROCESS: &str = r#"
INSERT OR REPLACE INTO gold_dim_process BY NAME
(
    SELECT
        pid,
        ppid,
        uid,
        command,
        full_command,
        started_at,
        CURRENT_TIMESTAMP AS inserted_at
    FROM
    (
        SELECT
            pro.pid,
            pro.ppid,
            pro.uid,
            ofn.command AS command,
            pro.command AS full_command,
            pro.lstart AS started_at,
            row_number() OVER (PARTITION BY pro.pid, pro.lstart ORDER BY pro.inserted_at DESC) AS row_num
        FROM silver_process_list pro
        LEFT JOIN silver_open_files ofn ON pro.pid = ofn.pid
    )
    WHERE ROW_NUM = 1
);"#;

const GOLD_DIM_FILE_REG: &str = r#"
INSERT INTO gold_dim_file_reg BY NAME
(
    SELECT
        pid,
        uid,
        fd,
        node,
        command,
        name,
        started_at,
        CURRENT_TIMESTAMP AS inserted_at
    FROM
        (
            SELECT
                pid,
                uid,
                fd,
                node,
                command,
                name,
                created_at AS started_at,
                ROW_NUMBER() OVER (PARTITION BY pid, fd, node ORDER BY created_at ASC) AS row_num
            FROM
                silver_open_files
            WHERE UPPER(type) NOT IN ('IPV4', 'IPV6')
        )
    WHERE row_num = 1
)
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at
;"#;

const GOLD_DIM_NETWORK_SOCKET: &str = r#"
INSERT INTO gold_dim_network_socket BY NAME
(
     SELECT
        HASH(pid, source_address, source_port, destination_address, destination_port) AS _id,
        pid,
        uid,
        command,
        source_address::INET AS source_address,
        source_port::USMALLINT AS source_port,
        destination_address,
        destination_port::USMALLINT AS destination_port,
        created_at AS started_at,
        CURRENT_TIMESTAMP AS inserted_at
    FROM
    (
        SELECT
        *
        FROM
        (
            SELECT
	            ofn.pid,
	            ofn.uid,
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
	                WHEN ofn.ip_destination_address IN ('*', '') THEN NULL
	                ELSE COALESCE(hos2.address, ofn.ip_destination_address)
	            END AS destination_address,
	            CASE
	                WHEN ofn.ip_destination_port IN ('*', '') THEN NULL
	                ELSE COALESCE(ser2.port::TEXT, ofn.ip_destination_port)
	            END AS destination_port,
	            ofn.created_at,
                ROW_NUMBER() OVER (PARTITION BY pid, ip_source_address, ip_source_port, ip_destination_address, ip_destination_port ORDER BY created_at ASC) AS row_num
            FROM silver_open_files ofn
	        LEFT JOIN gold_file_host hos1 ON LOWER(ofn.ip_source_address) = LOWER(hos1.name)
	        LEFT JOIN gold_file_host hos2 ON LOWER(ofn.ip_destination_address) = LOWER(hos2.name)
	        LEFT JOIN gold_file_service ser1 ON LOWER(ofn.ip_source_port) = LOWER(ser1.name)
	        LEFT JOIN gold_file_service ser2 ON LOWER(ofn.ip_destination_port) = LOWER(ser2.name)
            WHERE UPPER(ofn.type) IN ('IPV4', 'IPV6')
        )
        WHERE row_num = 1
    )
)
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at;
;"#;

const GOLD_DIM_NETWORK_OPEN_PORT: &str = r#"
INSERT INTO gold_dim_network_open_port BY NAME
(
    SELECT
        pid,
        uid,
        command,
        port::USMALLINT AS port,
        created_at AS started_at,
        CURRENT_TIMESTAMP AS inserted_at
    FROM
    (
        SELECT
            pid,
            uid,
            command,
            CASE
                WHEN sof.ip_source_port IN ('*', '') THEN NULL
                ELSE COALESCE(ser.port::TEXT, sof.ip_source_port)
            END AS port,
            created_at,
            ROW_NUMBER() OVER (PARTITION BY pid, port ORDER BY created_at ASC) AS row_num
        FROM silver_open_files sof
        LEFT JOIN gold_file_service ser ON LOWER(sof.ip_source_port) = LOWER(ser.name)
    )
    WHERE row_num = 1
    AND port IS NOT NULL
)
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at;
;"#;

const GOLD_DIM_NETWORK_LOCAL_IP: &str = r#"
WITH local_address AS
(
    SELECT DISTINCT
    *
    FROM
    (
        SELECT
            interface,
            address
        FROM silver_network_interface_address
        UNION ALL
        SELECT *
        FROM (
            VALUES
                (NULL, '255.255.255.255'::INET),
                (NULL, 'ff00::/8'::INET)
        ) AS cast_addr(interface, address)
    )
)
INSERT INTO gold_dim_network_local_ip BY NAME
(
SELECT DISTINCT
    HASH(ip.address::TEXT, adr.interface) AS _id,
    ip.address,
    adr.interface,
    ip.created_at AS started_at,
    CURRENT_TIMESTAMP AS inserted_at
FROM
(
    SELECT
        address,
        interface,
        created_at,
        FROM
        (
            SELECT
                *,
                ROW_NUMBER() OVER (PARTITION BY address, interface ORDER BY created_at ASC) AS row_num
            FROM
            (
                SELECT
                    source AS address,
                    interface,
                    created_at
                FROM silver_network_ip
                UNION ALL
                SELECT
                    destination AS address,
                    interface,
                    created_at
                FROM silver_network_ip
            )
        )
    WHERE row_num = 1
) ip
INNER JOIN local_address adr
ON ip.address <<= NETWORK(adr.address)
)
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at
;"#;

// TODO parse/add dns
const GOLD_DIM_NETWORK_FOREIGN_IP: &str = r#"
BEGIN;
WITH local_address AS
(
    SELECT DISTINCT
    *
    FROM
    (
        SELECT
            interface,
            address
        FROM silver_network_interface_address
        UNION ALL
        SELECT *
        FROM (
            VALUES
                (NULL, '255.255.255.255'::INET),
                (NULL, 'ff00::/8'::INET)
        ) AS cast_addr(interface, address)
    )
)
INSERT INTO gold_dim_network_foreign_ip BY NAME
(
SELECT DISTINCT
    HASH(ip.address::TEXT) AS _id,
    ip.address,
    NULL AS domain_name,
    ip.created_at AS started_at,
    CURRENT_TIMESTAMP AS inserted_at
FROM
(
    SELECT
        address,
        created_at,
        FROM
        (
            SELECT
                *,
                ROW_NUMBER() OVER (PARTITION BY address ORDER BY created_at ASC) AS row_num
            FROM
            (
                SELECT
                    source AS address,
                    created_at
                FROM silver_network_ip
                UNION ALL
                SELECT
                    destination AS address,
                    created_at
                FROM silver_network_ip
            )
        )
    WHERE row_num = 1
) ip
LEFT JOIN local_address adr ON ip.address <<= NETWORK(adr.address)
WHERE adr.address IS NULL
)
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at;
DELETE FROM gold_dim_network_foreign_ip
WHERE address IN (SELECT address FROM gold_dim_network_local_ip);
COMMIT;"#;

const GOLD_DIM_NETWORK_HOST: &str = r#"
INSERT OR REPLACE INTO gold_dim_network_host BY NAME
(
    SELECT DISTINCT
        _id,
        address,
        host,
        CURRENT_TIMESTAMP AS inserted_at
    FROM
    (
        SELECT
            HASH(source_address) AS _id,
            source_address AS address,
            HOST(source_address) AS host,
        FROM gold_dim_network_socket
        WHERE source_address IS NOT NULL
        UNION ALL
        SELECT
            HASH(address) AS _id,
            address,
            HOST(address) AS host,
        FROM gold_dim_network_local_ip
        UNION ALL
        SELECT
            HASH(address) AS _id,
            address,
            HOST(address) AS host,
        FROM gold_dim_network_foreign_ip
        UNION ALL
        SELECT
            HASH(source_address) AS _id,
            source_address,
            HOST(source_address) AS host,
        FROM gold_fact_network_ip
        UNION ALL
        SELECT
            HASH(destination_address) AS _id,
            destination_address,
            HOST(destination_address) AS host,
        FROM gold_fact_network_ip
    )
)
;"#;

const GOLD_FACT_PROCESS: &str = r#"
INSERT OR REPLACE INTO gold_fact_process BY NAME
(
SELECT DISTINCT
    pid,
    lstart AS started_at,
    created_at,
    pcpu,
    pmem,
    CURRENT_TIMESTAMP AS inserted_at
FROM silver_process_list
)
;"#;

const GOLD_FACT_FILE_REG: &str = r#"
INSERT OR REPLACE INTO gold_fact_file_reg BY NAME
(
SELECT DISTINCT
    pid,
    fd,
    node,
    created_at,
    size,
    CURRENT_TIMESTAMP AS inserted_at
FROM silver_open_files
WHERE UPPER(type) NOT IN ('IPV4', 'IPV6')
)
;"#;

const GOLD_FACT_NETWORK_PACKET: &str = r#"
INSERT OR REPLACE INTO gold_fact_network_packet BY NAME
(
    SELECT
        _id,
        interface,
        length,
        data_link,
        network,
        transport,
        application,
        created_at,
        CURRENT_TIMESTAMP AS inserted_at
    FROM silver_network_packet
);
"#;

const GOLD_FACT_NETWORK_IP: &str = r#"
INSERT OR REPLACE INTO gold_fact_network_ip BY NAME
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
        CURRENT_TIMESTAMP AS inserted_at
    FROM silver_network_transport transport
    INNER JOIN silver_network_ip ip ON transport._id = ip._id
    INNER JOIN silver_network_packet packet ON packet._id = ip._id
);
"#;

const GOLD_FACT_PROCESS_NETWORK: &str = r#"
INSERT OR REPLACE INTO gold_fact_process_network BY NAME
(
    WITH ip_packet AS
    (
        SELECT
            _id,
            created_at,
            source_address AS address,
            source_port AS port,
            TRUE AS send,
        FROM gold_fact_network_ip
        WHERE HOST(source_address) IN (SELECT address FROM gold_dim_network_local_ip)
        UNION
        SELECT
            _id,
            created_at,
            destination_address AS address,
            destination_port AS port,
            FALSE AS send,
        FROM gold_fact_network_ip
        WHERE HOST(destination_address) IN (SELECT address FROM gold_dim_network_local_ip)
    ),
    socket AS
    (
        SELECT DISTINCT pid, source_address AS address, source_port AS port, started_at, inserted_at
        FROM gold_dim_network_socket
        WHERE source_port IS NOT NULL
    )
    SELECT
        HASH(socket.pid, ip_packet._id, ip_packet.send) AS _id,
        socket.pid,
        ip_packet._id AS packet_id,
        ip_packet.send,
        CURRENT_TIMESTAMP AS inserted_at,
    FROM ip_packet
    INNER JOIN socket
    ON ip_packet.port = socket.port
    AND ip_packet.created_at >= socket.started_at
    AND ip_packet.created_at <= socket.inserted_at
);
"#;

const GOLD_TECH_TABLE_COUNT: &str = r#"
INSERT INTO gold_tech_table_count BY NAME
(
    SELECT 0 AS _id, 'bronze_process_list' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_process_list UNION
    SELECT 1 AS _id, 'bronze_open_files' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_open_files UNION
    SELECT 2 AS _id, 'bronze_network_packet' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_packet UNION
    SELECT 3 AS _id, 'bronze_network_ethernet' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_ethernet UNION
    SELECT 4 AS _id, 'bronze_network_interface_address' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_interface_address UNION
    SELECT 5 AS _id, 'bronze_network_ipv4' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_ipv4 UNION
    SELECT 6 AS _id, 'bronze_network_ipv6' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_ipv6 UNION
    SELECT 7 AS _id, 'bronze_network_arp' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_arp UNION
    SELECT 8 AS _id, 'bronze_network_tcp' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_tcp UNION
    SELECT 9 AS _id, 'bronze_network_udp' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_udp UNION
    SELECT 10 AS _id, 'bronze_network_icmp' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_icmp UNION
    SELECT 11 AS _id, 'bronze_network_tls' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_tls UNION
    SELECT 12 AS _id, 'bronze_network_http' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_http UNION
    SELECT 13 AS _id, 'bronze_network_dns_header' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_dns_header UNION
    SELECT 14 AS _id, 'bronze_network_dns_query' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_dns_query UNION
    SELECT 15 AS _id, 'bronze_network_dns_response' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM bronze_network_dns_response UNION
    SELECT 16 AS _id, 'silver_process_list' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM silver_process_list UNION
    SELECT 17 AS _id, 'silver_open_files' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM silver_open_files UNION
    SELECT 18 AS _id, 'silver_network_packet' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM silver_network_packet UNION
    SELECT 19 AS _id, 'silver_network_ethernet' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM silver_network_ethernet UNION
    SELECT 20 AS _id, 'silver_network_interface_address' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM silver_network_interface_address UNION
    SELECT 21 AS _id, 'silver_network_ip' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM silver_network_ip UNION
    SELECT 22 AS _id, 'silver_network_arp' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM silver_network_arp UNION
    SELECT 23 AS _id, 'silver_network_transport' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM silver_network_transport UNION
    SELECT 24 AS _id, 'silver_network_dns' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM silver_network_dns UNION
    SELECT 25 AS _id, 'gold_dim_process' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_dim_process UNION
    SELECT 26 AS _id, 'gold_dim_file_reg' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_dim_file_reg UNION
    SELECT 27 AS _id, 'gold_dim_network_socket' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_dim_network_socket UNION
    SELECT 28 AS _id, 'gold_dim_network_open_port' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_dim_network_open_port UNION
    SELECT 29 AS _id, 'gold_dim_network_local_ip' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_dim_network_local_ip UNION
    SELECT 30 AS _id, 'gold_dim_network_foreign_ip' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_dim_network_foreign_ip UNION
    SELECT 31 AS _id, 'gold_dim_network_host' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_dim_network_host UNION
    SELECT 32 AS _id, 'gold_fact_process' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_fact_process UNION
    SELECT 33 AS _id, 'gold_fact_file_reg' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_fact_file_reg UNION
    SELECT 34 AS _id, 'gold_fact_network_packet' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_fact_network_packet UNION
    SELECT 35 AS _id, 'gold_fact_network_ip' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_fact_network_ip UNION
    SELECT 36 AS _id, 'gold_fact_process_network' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_fact_process_network UNION
    SELECT 37 AS _id, 'gold_file_service' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_file_service UNION
    SELECT 38 AS _id, 'gold_file_host' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_file_host UNION
    SELECT 39 AS _id, 'gold_file_user' AS name, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM gold_file_user
)
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at,
    last_count = EXCLUDED.last_count,
    max_count = GREATEST(max_count, EXCLUDED.max_count)
;"#;

const GOLD_TECH_CHRONO: &str = r#"
INSERT INTO gold_tech_chrono BY NAME
(
    SELECT
        'process_list' AS name,
        EPOCH(MAX(brz_ingestion_duration)) AS brz_max_ingest,
        EPOCH(MIN(brz_ingestion_duration)) AS brz_min_ingest,
        EPOCH(MAX(svr_ingestion_duration)) AS svr_max_ingest,
        EPOCH(MIN(svr_ingestion_duration)) AS svr_min_ingest,
        EPOCH(MAX(brz_ingestion_duration) + MAX(svr_ingestion_duration)) AS max_ingest,
        EPOCH(MIN(brz_ingestion_duration) + MIN(svr_ingestion_duration)) AS min_ingest,
        CURRENT_TIMESTAMP AS inserted_at
    FROM silver_process_list
    UNION ALL
    SELECT
        'open_files' AS name,
        EPOCH(MAX(brz_ingestion_duration)) AS brz_max_ingest,
        EPOCH(MIN(brz_ingestion_duration)) AS brz_min_ingest,
        EPOCH(MAX(svr_ingestion_duration)) AS svr_max_ingest,
        EPOCH(MIN(svr_ingestion_duration)) AS svr_min_ingest,
        EPOCH(MAX(brz_ingestion_duration) + MAX(svr_ingestion_duration)) AS max_ingest,
        EPOCH(MIN(brz_ingestion_duration) + MIN(svr_ingestion_duration)) AS min_ingest,
        CURRENT_TIMESTAMP AS inserted_at
    FROM silver_open_files
    UNION ALL
    SELECT
        'network_packet' AS name,
        EPOCH(MAX(brz_ingestion_duration)) AS brz_max_ingest,
        EPOCH(MIN(brz_ingestion_duration)) AS brz_min_ingest,
        EPOCH(MAX(svr_ingestion_duration)) AS svr_max_ingest,
        EPOCH(MIN(svr_ingestion_duration)) AS svr_min_ingest,
        EPOCH(MAX(brz_ingestion_duration) + MAX(svr_ingestion_duration)) AS max_ingest,
        EPOCH(MIN(brz_ingestion_duration) + MIN(svr_ingestion_duration)) AS min_ingest,
        CURRENT_TIMESTAMP AS inserted_at
    FROM silver_network_packet
)
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at,
    brz_max_ingest = GREATEST(brz_max_ingest, EXCLUDED.brz_max_ingest),
    brz_min_ingest = LEAST(brz_min_ingest, EXCLUDED.brz_min_ingest),
    svr_max_ingest = GREATEST(svr_max_ingest, EXCLUDED.svr_max_ingest),
    svr_min_ingest = LEAST(svr_min_ingest, EXCLUDED.svr_min_ingest),
    max_ingest = GREATEST(max_ingest, EXCLUDED.max_ingest),
    min_ingest = LEAST(min_ingest, EXCLUDED.min_ingest)
;"#;

pub fn request() -> String {
    format!(
        "{} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        GOLD_DIM_PROCESS,
        GOLD_DIM_FILE_REG,
        GOLD_DIM_NETWORK_SOCKET,
        GOLD_DIM_NETWORK_OPEN_PORT,
        GOLD_DIM_NETWORK_LOCAL_IP,
        GOLD_DIM_NETWORK_FOREIGN_IP,
        GOLD_DIM_NETWORK_HOST,
        GOLD_FACT_PROCESS,
        GOLD_FACT_FILE_REG,
        GOLD_FACT_NETWORK_PACKET,
        GOLD_FACT_NETWORK_IP,
        GOLD_FACT_PROCESS_NETWORK,
        GOLD_TECH_TABLE_COUNT,
        GOLD_TECH_CHRONO
    )
}
