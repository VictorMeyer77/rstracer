use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("IO Error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Regex: {0}")]
    Regex(#[from] regex::Error),
}
