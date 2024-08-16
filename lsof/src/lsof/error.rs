use crate::lsof::OpenFile;
use std::io;
use std::num;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("lsof is not implemented for OS {os:} with architecture {arch:}.")]
    Unimplemented { os: String, arch: String },
    #[error("Error parsing row: {row:}")]
    ParseRow { row: String },
    #[error("Error parsing integer: {0}")]
    ParseInt(#[from] num::ParseIntError),
    #[error("Error parsing float: {0}")]
    ParseFloat(#[from] num::ParseFloatError),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Channel error: {0}")]
    Channel(#[from] SendError<OpenFile>),
}
