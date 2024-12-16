use std::io;
use std::num;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing integer: {0}")]
    ParseInt(#[from] num::ParseIntError),
    #[error("Error parsing float: {0}")]
    ParseFloat(#[from] num::ParseFloatError),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
}
