use crate::Log;
use std::io;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("lsof is not implemented for OS {os:}.")]
    Unimplemented { os: String },
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Channel error: {0}")]
    Channel(#[from] SendError<Log>),
}
