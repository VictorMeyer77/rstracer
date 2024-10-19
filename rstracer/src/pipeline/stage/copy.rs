use crate::pipeline::error::Error;
use crate::pipeline::stage::schema::Schema;

fn copy_table_request(
    source: &str,
    target: &str,
    table_name: &str,
    columns: &str,
    overwrite: bool,
) -> String {
    let mut request = String::new();
    if overwrite {
        request.push_str(&format!("TRUNCATE {target}.{table_name};"));
    }
    request.push_str(
    &format!("INSERT INTO {target}.{table_name} ({columns}) SELECT {columns} FROM {source}.{table_name};")
    );
    request
}

pub fn copy_layer_request(
    schema: &Schema,
    source: &str,
    target: &str,
    layer: &str,
    overwrite: bool,
) -> Result<String, Error> {
    let mut query: String = String::new();
    for table in &schema.tables {
        let columns: Vec<String> = table
            .columns
            .iter()
            .filter(|c| !c.is_autoincrement)
            .map(|c| c.name.clone())
            .collect();

        if table.name.starts_with(layer) {
            query.push_str(&copy_table_request(
                source,
                target,
                &table.name,
                &columns.join(","),
                overwrite,
            ))
        }
    }
    Ok(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::stage::schema::tests::create_mock_schema;

    #[test]
    fn test_copy_layer_request_without_overwrite() {
        let schema = create_mock_schema();
        let source = "source_db";
        let target = "target_db";
        let layer = "bronze";
        let overwrite = false;

        let result = copy_layer_request(&schema, source, target, layer, overwrite).unwrap();
        let expected = "INSERT INTO target_db.bronze_process_list (pid,inserted_at) SELECT pid,inserted_at FROM source_db.bronze_process_list;";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_copy_layer_request_with_overwrite() {
        let schema = create_mock_schema();
        let source = "source_db";
        let target = "target_db";
        let layer = "silver";
        let overwrite = true;

        let result = copy_layer_request(&schema, source, target, layer, overwrite).unwrap();
        let expected = "TRUNCATE target_db.silver_process_list;INSERT INTO target_db.silver_process_list (pid,inserted_at) SELECT pid,inserted_at FROM source_db.silver_process_list;";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_copy_layer_request_empty_schema() {
        let schema = Schema { tables: vec![] };
        let source = "source_db";
        let target = "target_db";
        let layer = "bronze";
        let overwrite = false;

        let result = copy_layer_request(&schema, source, target, layer, overwrite).unwrap();
        let expected = "";

        assert_eq!(result, expected);
    }
}
