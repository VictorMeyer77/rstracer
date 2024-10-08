use crate::config::VacuumConfig;
use crate::pipeline::stage::schema::Schema;

pub fn request(config: VacuumConfig, schema: Schema) -> String {
    let mut query: String = "".to_string();

    for table in schema.tables {
        for layer in config.to_list() {
            if table.name.starts_with(&layer.0) && layer.1 > 0 {
                query.push_str(&format!(
                    "DELETE FROM memory.{} WHERE inserted_at + '{} seconds' < CURRENT_TIMESTAMP;",
                    table.name, layer.1
                ));
            }
        }
    }

    query
}
