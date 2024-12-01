use crate::config::ExportConfig;
use crate::pipeline::stage::schema::get_schema;
use std::fs;

fn table_export_request(table_name: &str, directory: &str, format: &str) -> String {
    format!("COPY {table_name} TO '{directory}/{table_name}.{format}' (FORMAT {format});")
}

pub fn request(config: &ExportConfig) -> String {
    if !["parquet", "csv"].contains(&config.format.to_lowercase().as_str()) {
        panic!("Unsupported export format: {}", &config.format)
    }
    fs::create_dir_all(&config.directory).unwrap();
    let schema = get_schema();
    let requests: Vec<String> = schema
        .into_iter()
        .filter(|table| table.starts_with("gold_"))
        .map(|table| table_export_request(&table, &config.directory, &config.format))
        .collect();
    requests.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::stage::tests::create_test_connection;
    use std::fs;

    #[test]
    fn test_table_export_request_parquet() {
        let table_name = "user_gold_data";
        let directory = "/exports";
        let format = "parquet";

        let result = table_export_request(table_name, directory, format);
        let expected = "COPY user_gold_data TO '/exports/user_gold_data.parquet' (FORMAT parquet);";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_table_export_request_csv() {
        let table_name = "user_gold_data";
        let directory = "/exports";
        let format = "csv";

        let result = table_export_request(table_name, directory, format);
        let expected = "COPY user_gold_data TO '/exports/user_gold_data.csv' (FORMAT csv);";

        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "Unsupported export format: json")]
    fn test_request_with_unsupported_format() {
        let config = ExportConfig {
            format: "json".to_string(),
            directory: "/exports".to_string(),
        };
        request(&config);
    }

    #[test]
    fn test_request_parquet() {
        let test_path = "target/test/parquet/";
        let connection = create_test_connection();
        let config = ExportConfig {
            format: "parquet".to_string(),
            directory: test_path.to_string(),
        };
        let request = request(&config);

        connection.execute_batch(&request).unwrap();
        let count = fs::read_dir(test_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .count();

        assert_eq!(count, 17);
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_request_csv() {
        let test_path = "target/test/csv/";
        let connection = create_test_connection();
        let config = ExportConfig {
            format: "csv".to_string(),
            directory: test_path.to_string(),
        };
        let request = request(&config);

        connection.execute_batch(&request).unwrap();
        let count = fs::read_dir(test_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .count();

        assert_eq!(count, 17);
        fs::remove_dir_all(test_path).unwrap();
    }
}
