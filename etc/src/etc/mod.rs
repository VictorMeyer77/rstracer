pub mod error;
pub mod service;

use error::Error;

pub trait EtcReader<T> {
    fn read_etc_file(path: Option<&str>) -> Result<Vec<T>, Error>;
}
