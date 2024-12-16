use procfs::ProcError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Proc: {0}")]
    Proc(#[from] ProcError),
}
