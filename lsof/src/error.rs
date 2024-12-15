use crate::OpenFile;
use std::io;
use std::num;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing integer: {0}")]
    ParseInt(#[from] num::ParseIntError),
    #[error("Error parsing float: {0}")]
    ParseFloat(#[from] num::ParseFloatError),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Channel error with socket: {0}")]
    Channel(#[from] Box<SendError<OpenFile>>),
}
