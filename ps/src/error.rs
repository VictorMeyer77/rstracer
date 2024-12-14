use crate::Process;
use chrono;
use std::io;
use std::num;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing date: {0}")]
    ParseDate(#[from] chrono::ParseError),
    #[error("Error parsing integer: {0}")]
    ParseInt(#[from] num::ParseIntError),
    #[error("Error parsing float: {0}")]
    ParseFloat(#[from] num::ParseFloatError),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("{msg}")]
    Stdout { msg: String },
    #[error("Channel error with process: {0}")]
    Channel(#[from] Box<SendError<Process>>),
}
