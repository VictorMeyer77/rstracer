use crate::pipeline::error::Error;
use config::File;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub disk_file_path: String,
    pub request: ChannelConfig,
    pub ps: ChannelConfig,
    pub persist_layer: CopyConfig,
    pub load_layer: CopyConfig,
    pub vacuum: VacuumConfig,
    pub schedule: ScheduleConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChannelConfig {
    pub channel_size: usize,
    pub producer_frequency: Option<u64>,
    pub consumer_batch_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CopyConfig {
    pub bronze: bool,
    pub silver: bool,
    pub gold: bool,
}

impl CopyConfig {
    pub fn to_list(&self) -> Vec<(String, bool)> {
        vec![
            ("bronze".to_string(), self.bronze),
            ("silver".to_string(), self.silver),
            ("gold".to_string(), self.gold),
        ]
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct VacuumConfig {
    pub bronze: u64,
    pub silver: u64,
    pub gold: u64,
}

impl VacuumConfig {
    pub fn to_list(&self) -> Vec<(String, u64)> {
        vec![
            ("bronze".to_string(), self.bronze),
            ("silver".to_string(), self.silver),
            ("gold".to_string(), self.gold),
        ]
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleConfig {
    pub silver: u64,
    pub gold: u64,
    pub vacuum: u64,
}

impl ScheduleConfig {
    pub fn to_list(&self) -> Vec<(String, u64)> {
        vec![
            ("silver".to_string(), self.silver),
            ("gold".to_string(), self.gold),
            ("vacuum".to_string(), self.vacuum),
        ]
    }
}

pub fn read_config() -> Result<Config, Error> {
    let config = config::Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()?;
    Ok(config.try_deserialize()?)
}
