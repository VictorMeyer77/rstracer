use crate::Socket;
use procfs::ProcError;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Channel error with socket: {0}")]
    Channel(#[from] Box<SendError<Socket>>),
    #[error("Proc: {0}")]
    Proc(#[from] ProcError),
}
