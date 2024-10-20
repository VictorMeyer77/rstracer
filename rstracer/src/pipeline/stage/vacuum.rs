use crate::config::VacuumConfig;
use crate::pipeline::stage::schema::Schema;

pub fn request(config: VacuumConfig, schema: Schema) -> String {
    let mut query: String = "".to_string();

    for table in schema.tables {
        for layer in config.to_list() {
            if table.name.starts_with(&layer.0)
                && layer.1 > 0
                && !table.name.contains("_dim_")
                && !table.name.contains("_tech_")
            {
                query.push_str(&format!(
                    "BEGIN; DELETE FROM memory.{} WHERE inserted_at + '{} seconds' < CURRENT_TIMESTAMP; COMMIT;",
                    table.name, layer.1
                ));
            }
        }
    }

    query
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::stage::schema::tests::create_mock_schema;

    #[test]
    fn test_vacuum_request() {
        let schema = create_mock_schema();
        let vacuum_config = VacuumConfig {
            bronze: 15,
            silver: 30,
            gold: 1000,
        };
        let request = request(vacuum_config, schema);
        assert_eq!(
            request,
            r#"BEGIN; DELETE FROM memory.bronze_process_list WHERE inserted_at + '15 seconds' < CURRENT_TIMESTAMP; COMMIT;BEGIN; DELETE FROM memory.silver_process_list WHERE inserted_at + '30 seconds' < CURRENT_TIMESTAMP; COMMIT;BEGIN; DELETE FROM memory.gold_process_list WHERE inserted_at + '1000 seconds' < CURRENT_TIMESTAMP; COMMIT;"#
        );
    }
}
