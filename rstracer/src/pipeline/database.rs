use crate::pipeline::error::Error;
use crate::pipeline::stage::copy::copy_layer_request;
use crate::pipeline::stage::schema::{Column, Schema, Table, GET_SCHEMA};
use duckdb::{Connection, Row};
use lazy_static::lazy_static;
use std::sync::Mutex;
use tracing::info;

lazy_static! {
    static ref DATABASE: Mutex<Connection> = Mutex::new(Connection::open(":memory:").unwrap());
}

pub fn execute_request(request: &str) -> Result<(), Error> {
    Ok(DATABASE.lock().unwrap().execute_batch(request)?)
}

pub fn get_schema() -> Result<Schema, Error> {
    let connection = DATABASE.lock().unwrap();
    let mut statement = connection.prepare(GET_SCHEMA)?;
    let mut rows = statement.query([])?;
    let mut table_buffer: Vec<Table> = vec![];
    while let Some(row) = rows.next()? {
        table_buffer.push(row_to_schema_table(row)?)
    }
    Ok(Schema {
        tables: table_buffer,
    })
}

fn row_to_schema_table(row: &Row<'_>) -> Result<Table, Error> {
    let table_name: String = row.get(0)?;
    let column_names: String = row.get(1)?;
    let column_names: Vec<String> = column_names.split(';').map(|c| c.to_string()).collect();
    let column_types: String = row.get(2)?;
    let column_types: Vec<String> = column_types.split(';').map(|c| c.to_string()).collect();
    let column_nullables: String = row.get(3)?;
    let column_nullables: Vec<bool> = column_nullables.split(';').map(|c| c == "true").collect();
    let column_autoincrement: String = row.get(4)?;
    let column_autoincrement: Vec<bool> = column_autoincrement
        .split(';')
        .map(|c| c == "true")
        .collect();
    let zip = column_names
        .into_iter()
        .zip(column_types)
        .zip(column_nullables)
        .zip(column_autoincrement)
        .map(|(((a, b), c), d)| (a, b, c, d));
    let columns: Vec<Column> = zip
        .map(|(name, _type, null, auto)| Column {
            name,
            type_name: _type,
            is_nullable: null,
            is_autoincrement: auto,
        })
        .collect();
    Ok(Table {
        name: table_name,
        columns,
    })
}

pub fn copy_layer(
    schema: &Schema,
    source: &str,
    target: &str,
    layers: Vec<(String, bool)>,
) -> Result<(), Error> {
    for layer in layers {
        if layer.1 {
            info!("copy layer {} from {source} to {target}...", layer.0);
            execute_request(&copy_layer_request(schema, source, target, &layer.0, true)?)?
        }
    }
    Ok(())
}
