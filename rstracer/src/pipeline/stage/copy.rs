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
