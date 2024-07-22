use std::thread::sleep;
use std::time::Duration;
use polars::frame::DataFrame;
use std::io::Write;

pub mod error;
pub mod source;

pub trait BronzeCompute {
    fn extract(&mut self) -> DataFrame;
    fn transform(df: &mut DataFrame) -> ();
    fn load(df: DataFrame, storage: &mut BronzeStorage) -> ();
    fn launch(&mut self, storage: &mut BronzeStorage) {
        loop {
            let mut df= self.extract();
            Self::transform(&mut df);
            Self::load(df, storage);
            sleep(Duration::from_millis(1000));
        }
    }
}

pub struct BronzeStorage {
    pub process_df: Option<DataFrame>
}

impl BronzeStorage {
    pub fn new() -> BronzeStorage {
        BronzeStorage {
            process_df: None
        }
    }
}