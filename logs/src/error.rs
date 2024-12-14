use crate::JournalLog;
use std::num;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Regex: {0}")]
    Regex(#[from] regex::Error),
    #[error("Parsing integer: {0}")]
    ParseInt(#[from] num::ParseIntError),
    #[error("Parsing float: {0}")]
    ParseFloat(#[from] num::ParseFloatError),
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("Channel error with log: {0}")]
    Channel(#[from] Box<SendError<JournalLog>>),
    #[error("{msg}")]
    Stdout { msg: String },
}
