use crate::config::VacuumConfig;
use crate::pipeline::stage::schema::get_schema;

pub fn request(config: &VacuumConfig) -> String {
    let mut query: String = String::new();

    for table in get_schema() {
        for layer in config.to_list() {
            if table.starts_with(&layer.0)
                && layer.1 > 0
                && !table.contains("gold_file_")
                && !table.contains("_tech_")
            {
                query.push_str(&format!(
                    "DELETE FROM {} WHERE inserted_at + '{} seconds' < CURRENT_TIMESTAMP;",
                    table, layer.1
                ));
            }
        }
    }
    query
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vacuum_request() {
        let vacuum_config = VacuumConfig {
            bronze: 15,
            silver: 30,
            gold: 1000,
        };
        let request = request(&vacuum_config);
        println!("{:?}", request);
        assert!(!request.contains("gold_file_"));
        assert!(!request.contains("_tech_"));
        assert_eq!(request.matches("DELETE FROM").count(), 36);
        assert!(request.contains(
            "DELETE FROM bronze_process_list WHERE inserted_at + '15 seconds' < CURRENT_TIMESTAMP"
        ));
        assert!(request.contains(
            "DELETE FROM silver_process_list WHERE inserted_at + '30 seconds' < CURRENT_TIMESTAMP"
        ));
        assert!(request.contains(
            "DELETE FROM gold_fact_process WHERE inserted_at + '1000 seconds' < CURRENT_TIMESTAMP"
        ));
    }

    #[test]
    fn test_vacuum_request_with_permanent() {
        let vacuum_config = VacuumConfig {
            bronze: 15,
            silver: 30,
            gold: 0,
        };
        let request = request(&vacuum_config);
        assert!(!request.contains("gold_"));
        assert_eq!(request.matches("DELETE FROM").count(), 25);
        assert!(request.contains(
            "DELETE FROM bronze_process_list WHERE inserted_at + '15 seconds' < CURRENT_TIMESTAMP"
        ));
        assert!(request.contains(
            "DELETE FROM silver_process_list WHERE inserted_at + '30 seconds' < CURRENT_TIMESTAMP"
        ));
    }
}
