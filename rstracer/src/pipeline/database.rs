use crate::pipeline::error::Error;
use duckdb::Connection;
use lazy_static::lazy_static;
use std::sync::Mutex;
use tracing::info;

const DATABASE_FILE_PATH: &str = "rstracer.db";

lazy_static! {
    static ref DATABASE_MEMORY: Mutex<Connection> =
        Mutex::new(Connection::open_in_memory().unwrap());
}

lazy_static! {
    static ref DATABASE_FILE: Mutex<Connection> =
        Mutex::new(Connection::open(DATABASE_FILE_PATH).unwrap());
}

pub fn execute_request(request: &str, in_memory: bool) -> Result<(), Error> {
    if in_memory {
        Ok(DATABASE_MEMORY.lock().unwrap().execute_batch(request)?)
    } else {
        Ok(DATABASE_FILE.lock().unwrap().execute_batch(request)?)
    }
}

pub fn close_connection(in_memory: bool) -> Result<(), Error> {
    execute_request("CHECKPOINT;", in_memory)?;
    info!("database connection close gracefully");
    Ok(())
}
