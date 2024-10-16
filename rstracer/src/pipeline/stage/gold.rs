const GOLD_PROCESS_LIST: &str = r#"
INSERT INTO memory.gold_process_list BY NAME
(
    SELECT
        pid,
        ppid,
        uid,
        command,
        MIN(pcpu) OVER (PARTITION BY pid, lstart ORDER BY row_num) AS min_pcpu,
        MAX(pcpu) OVER (PARTITION BY pid, lstart ORDER BY row_num) AS max_pcpu,
        pcpu AS last_pcpu,
        MIN(pmem) OVER (PARTITION BY pid, lstart ORDER BY row_num) AS min_pmem,
        MAX(pmem) OVER (PARTITION BY pid, lstart ORDER BY row_num) AS max_pmem,
        pmem AS last_pmem,
        _id AS silver_id,
        lstart AS started_at,
        CURRENT_TIMESTAMP AS inserted_at
    FROM
    (
        SELECT
            *,
            row_number() OVER (PARTITION BY pid, lstart ORDER BY inserted_at DESC) AS row_num
        FROM memory.silver_process_list
    )
    WHERE ROW_NUM = 1
)
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at,
    last_pcpu = EXCLUDED.last_pcpu,
    min_pcpu = LEAST(min_pcpu, EXCLUDED.min_pcpu),
    max_pcpu = GREATEST(max_pcpu, EXCLUDED.max_pcpu),
    last_pmem = EXCLUDED.last_pmem,
    min_pmem = LEAST(min_pmem, EXCLUDED.min_pmem),
    max_pmem = GREATEST(max_pmem, EXCLUDED.max_pmem)
;"#;

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
        MIN(size) OVER (PARTITION BY pid, fd, node ORDER BY row_num) AS min_size,
        MAX(size) OVER (PARTITION BY pid, fd, node ORDER BY row_num) AS max_size,
        SIZE AS last_size,
        _id AS silver_id,
        created_at AS started_at,
        CURRENT_TIMESTAMP AS inserted_at
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
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at,
    last_size = EXCLUDED.last_size,
    min_size = LEAST(min_size, EXCLUDED.min_size),
    max_size = GREATEST(max_size, EXCLUDED.max_size)
;"#;

const GOLD_OPEN_FILES_NETWORK: &str = r#"
INSERT OR REPLACE INTO memory.gold_open_files_network BY NAME
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
        CURRENT_TIMESTAMP AS inserted_at
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
);"#;

const GOLD_NETWORK_PACKET: &str = r#"
INSERT OR REPLACE INTO memory.gold_network_packet BY NAME
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
    FROM memory.silver_network_packet
);
"#;

const GOLD_NETWORK_IP: &str = r#"
INSERT OR REPLACE INTO memory.gold_network_ip BY NAME
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
    FROM memory.silver_network_transport transport
    INNER JOIN memory.silver_network_ip ip ON transport._id = ip._id
    INNER JOIN memory.silver_network_packet packet ON packet._id = ip._id
);
"#;

const GOLD_PROCESS_NETWORK: &str = r#"
INSERT OR REPLACE INTO memory.gold_process_network BY NAME
(
    SELECT DISTINCT
        HASH(pro.silver_id, ofn.silver_id, net._id) AS _id,
        pro.pid,
        pro.uid,
        ofn.command,
        net.source_address,
        net.source_port,
        net.destination_address,
        net.destination_port,
        net.is_source,
        pro.silver_id AS process_svr_id,
        ofn.silver_id AS open_file_svr_id,
        net._id AS packet_id,
        CURRENT_TIMESTAMP AS inserted_at,
    FROM memory.gold_process_list pro
    INNER JOIN memory.gold_open_files_network ofn
    ON pro.pid = ofn.pid
    AND ofn.started_at > pro.started_at
    AND ofn.started_at < pro.inserted_at
    LEFT JOIN (
        SELECT
            _id,
            ip_version,
            transport_protocol,
            source_address,
            source_port,
            destination_address,
            destination_port,
            created_at,
            inserted_at,
            source_address AS address_key,
            source_port AS port_key,
            TRUE AS is_source,
        FROM memory.gold_network_ip
        UNION
        SELECT
            _id,
            ip_version,
            transport_protocol,
            source_address,
            source_port,
            destination_address,
            destination_port,
            created_at,
            inserted_at,
            destination_address AS address_key,
            destination_port AS port_key,
            FALSE AS is_source,
        FROM memory.gold_network_ip
    ) net
    ON host(net.address_key) = host(ofn.source_address)
    AND net.port_key = ofn.source_port
    AND net.created_at > ofn.started_at
    AND net.created_at < ofn.inserted_at
);
"#;

const GOLD_PROCESS_COMMAND: &str = r#"
INSERT OR REPLACE INTO memory.gold_process_command BY NAME
(
    SELECT DISTINCT
        pro.pid,
        pro.ppid,
        ofn.command,
        CURRENT_TIMESTAMP AS inserted_at
    FROM memory.silver_process_list pro
    LEFT JOIN memory.silver_open_files ofn ON pro.pid = ofn.pid
);
"#;

const GOLD_TECH_TABLE_COUNT: &str = r#"
INSERT INTO memory.gold_tech_table_count BY NAME
(
    SELECT 0 AS _id, 'bronze_process_list' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_process_list UNION
    SELECT 1 AS _id, 'bronze_open_files' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_open_files UNION
    SELECT 2 AS _id, 'bronze_network_packet' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_packet UNION
    SELECT 3 AS _id, 'bronze_network_ethernet' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_ethernet UNION
    SELECT 4 AS _id, 'bronze_network_interface_address' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_interface_address UNION
    SELECT 5 AS _id, 'bronze_network_ipv4' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_ipv4 UNION
    SELECT 6 AS _id, 'bronze_network_ipv6' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_ipv6 UNION
    SELECT 7 AS _id, 'bronze_network_arp' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_arp UNION
    SELECT 8 AS _id, 'bronze_network_tcp' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_tcp UNION
    SELECT 9 AS _id, 'bronze_network_udp' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_udp UNION
    SELECT 10 AS _id, 'bronze_network_icmp' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_icmp UNION
    SELECT 11 AS _id, 'bronze_network_tls' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_tls UNION
    SELECT 12 AS _id, 'bronze_network_http' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_http UNION
    SELECT 13 AS _id, 'bronze_network_dns_header' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_dns_header UNION
    SELECT 14 AS _id, 'bronze_network_dns_query' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_dns_query UNION
    SELECT 15 AS _id, 'bronze_network_dns_response' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.bronze_network_dns_response UNION
    SELECT 16 AS _id, 'silver_process_list' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.silver_process_list UNION
    SELECT 17 AS _id, 'silver_open_files' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.silver_open_files UNION
    SELECT 18 AS _id, 'silver_network_packet' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.silver_network_packet UNION
    SELECT 19 AS _id, 'silver_network_ethernet' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.silver_network_ethernet UNION
    SELECT 20 AS _id, 'silver_network_interface_address' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.silver_network_interface_address UNION
    SELECT 21 AS _id, 'silver_network_ip' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.silver_network_ip UNION
    SELECT 22 AS _id, 'silver_network_arp' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.silver_network_arp UNION
    SELECT 23 AS _id, 'silver_network_transport' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.silver_network_transport UNION
    SELECT 24 AS _id, 'silver_network_dns' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.silver_network_dns UNION
    SELECT 25 AS _id, 'gold_process_list' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.gold_process_list UNION
    SELECT 26 AS _id, 'gold_open_files_regular' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.gold_open_files_regular UNION
    SELECT 27 AS _id, 'gold_open_files_network' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.gold_open_files_network UNION
    SELECT 28 AS _id, 'gold_network_packet' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.gold_network_packet UNION
    SELECT 29 AS _id, 'gold_network_ip' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.gold_network_ip UNION
    SELECT 30 AS _id, 'gold_process_command' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.gold_process_command UNION
    SELECT 31 AS _id, 'gold_process_network' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.gold_process_network UNION
    SELECT 32 AS _id, 'gold_dim_hosts' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.gold_dim_hosts UNION
    SELECT 33 AS _id, 'gold_dim_services' AS name, count(*) AS min_count, count(*) AS max_count, count(*) AS last_count, CURRENT_TIMESTAMP AS inserted_at FROM memory.gold_dim_services
)
ON CONFLICT DO UPDATE SET
    inserted_at = EXCLUDED.inserted_at,
    last_count = EXCLUDED.last_count,
    min_count = LEAST(min_count, EXCLUDED.min_count),
    max_count = GREATEST(max_count, EXCLUDED.max_count)
;"#;

const GOLD_TECH_CHRONO: &str = r#"
INSERT INTO memory.gold_tech_chrono BY NAME
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
    FROM memory.silver_process_list
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
    FROM memory.silver_open_files
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
    FROM memory.silver_network_packet
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
        "{} {} {} {} {} {} {} {} {}",
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
