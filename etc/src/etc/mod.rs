pub mod error;
pub mod host;
pub mod service;
pub mod user;

use error::Error;

pub trait EtcReader<T> {
    fn read_etc_file(path: Option<&str>) -> Result<Vec<T>, Error>;
}
