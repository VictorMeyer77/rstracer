use config::ConfigError;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
    #[error("Database error: {0}")]
    Database(#[from] duckdb::Error),
    #[error("Channel error: {0}")]
    Channel(#[from] Box<SendError<String>>),
    #[error("Join error: {0}")]
    Join(#[from] JoinError),
    #[error("Etc error: {0}")]
    EtcError(#[from] etc::etc::error::Error),
}
