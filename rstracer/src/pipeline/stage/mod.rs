pub mod bronze;
pub mod copy;
pub mod dimension;
pub mod gold;
pub mod schema;
pub mod silver;
pub mod vacuum;

#[cfg(test)]
pub mod tests {
    use crate::pipeline::stage::schema::create_schema_request;
    use duckdb::Connection;

    pub fn create_test_connection() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(&create_schema_request("test.db"))
            .unwrap();
        connection
    }
}
