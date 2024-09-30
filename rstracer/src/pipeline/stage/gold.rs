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
            row_number() OVER (PARTITION BY address ORDER BY inserted_at) AS row_num
            FROM
            (
                SELECT source AS address, version, inserted_at
                FROM silver_network_ip
                UNION ALL
                SELECT destination AS address, version, inserted_at
                FROM silver_network_ip
            )
        )
    WHERE row_num = 1
);
"#;

pub fn request() -> String {
    format!("{}", GOLD_NETWORK_IP)
}
