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

const SILVER_NETWORK_DNS: &str = r#"
INSERT OR IGNORE INTO memory.silver_network_dns BY NAME
(
SELECT
    CONCAT_WS('-', CAST(header._id AS TEXT), CAST(query._id AS VARCHAR), CAST(response._id AS VARCHAR)) AS _id,
    header.packet_id,
    header.id,
    header.is_response,
    header.opcode,
    header.is_authoriative,
    header.is_truncated,
    header.is_recursion_desirable,
    header.is_recursion_available,
    header.zero_reserved,
    header.is_answer_authenticated,
    header.is_non_authenticated_data,
    header.rcode,
    header.query_count,
    header.response_count,
    header.authority_rr_count,
    header.additional_rr_count,
    query.qname,
    query.qtype,
    query.qclass,
    response.origin,
    response.name_tag,
    response.rtype,
    response.rclass,
    response.ttl,
    response.rdlength,
    response.rdata,
    packet.created_at,
    packet.brz_ingestion_duration,

    ARRAY_TO_STRING(
        LIST_TRANSFORM(query.qname, (c, i) ->
            CASE
                WHEN (c IN (45, 95, 32)) OR
                     (c > 47 AND c < 58) OR
                     (c > 64 AND c < 91) OR
                     (c > 96 AND c < 123) THEN
                    CHR(c)
                ELSE
                    CASE
                        WHEN i = 1 OR i = LENGTH(query.qname) THEN ''
                        ELSE '.'
                    END
            END
        ),
        ''
    ) AS question_parsed,

    CASE
        WHEN (response.rclass = 'IN') AND (response.rtype IN ('A', 'AAAA')) THEN
            REPLACE(
                REPLACE(
                    REPLACE(
                        CAST(response.rdata AS TEXT),
                        ', ', '.'
                    ),
                    '[', ''
                ),
                ']', ''
            )
        ELSE
            ARRAY_TO_STRING(
                LIST_TRANSFORM(response.rdata, (c, i) ->
                    CASE
                        WHEN (c IN (45, 95, 32)) OR
                             (c > 47 AND c < 58) OR
                             (c > 64 AND c < 91) OR
                             (c > 96 AND c < 123) THEN
                            CHR(c)
                        ELSE
                            CASE
                                WHEN i = 1 OR i = LENGTH(query.qname) THEN ''
                                ELSE '.'
                            END
                    END
                ),
                ''
            )
    END AS response_parsed,

    CURRENT_TIMESTAMP AS inserted_at,
    AGE(packet.inserted_at) AS svr_ingestion_duration

FROM
    bronze_network_dns_header header
LEFT JOIN
    bronze_network_dns_query query ON header.packet_id = query.packet_id
LEFT JOIN
    bronze_network_dns_response response ON header.packet_id = response.packet_id
LEFT JOIN
	bronze_network_packet packet ON header.packet_id = packet._id
);
"#;

const SILVER_NETWORK_IP: &str = r#"
INSERT OR IGNORE INTO memory.silver_network_ip BY NAME
(
SELECT
    ip.*,
    packet.created_at,
    packet.brz_ingestion_duration,
    CURRENT_TIMESTAMP AS inserted_at,
    AGE(packet.inserted_at) AS svr_ingestion_duration
FROM
(
    SELECT
        packet_id AS _id,
        version,
        total_length AS length,
        ttl AS hop_limit,
        next_level_protocol AS next_protocol,
        source,
        destination
    FROM memory.bronze_network_ipv4
    UNION ALL
    SELECT
        packet_id AS _id,
        version,
        payload_length AS length,
        hop_limit,
        next_header AS next_protocol,
        source,
        destination
    FROM memory.bronze_network_ipv6
) ip LEFT JOIN memory.bronze_network_packet packet ON ip._id = packet._id
);
"#;

const SILVER_NETWORK_TRANSPORT: &str = r#"
INSERT OR IGNORE INTO memory.silver_network_transport BY NAME
(
SELECT
    transport.*,
    packet.length AS packet_length,
    packet.interface AS interface,
    packet.created_at,
    packet.brz_ingestion_duration,
    CURRENT_TIMESTAMP AS inserted_at,
    AGE(packet.inserted_at) AS svr_ingestion_duration
FROM
(
    SELECT
        packet_id AS _id,
        'TCP' AS protocol,
        source,
        destination
    FROM bronze_network_tcp
	UNION ALL
	SELECT
		packet_id AS _id,
		'UDP' AS protocol,
		source,
		destination
	FROM bronze_network_udp
) transport LEFT JOIN bronze_network_packet packet ON transport._id = packet._id
);
"#;

pub fn silver_request() -> String {
    format!(
        "{} {} {} {} {}",
        SILVER_PROCESS_LIST,
        SILVER_OPEN_FILES,
        SILVER_NETWORK_DNS,
        SILVER_NETWORK_IP,
        SILVER_NETWORK_TRANSPORT
    )
}
