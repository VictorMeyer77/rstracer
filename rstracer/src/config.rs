use crate::pipeline::error::Error;
use config;
use serde::Deserialize;
use std::path::Path;

const CONFIG_FILE_PATH: &str = "rstracer.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub disk_file_path: String,
    pub request: ChannelConfig,
    pub ps: ChannelConfig,
    pub lsof: ChannelConfig,
    pub network: ChannelConfig,
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
    let mut config = config::Config::builder()
        .set_default("disk_file_path", "/tmp/rstracer.db")?
        // persist_layer
        .set_default("persist_layer.bronze", true)?
        .set_default("persist_layer.silver", true)?
        .set_default("persist_layer.gold", true)?
        // load_layer
        .set_default("load_layer.bronze", false)?
        .set_default("load_layer.silver", false)?
        .set_default("load_layer.gold", true)?
        // vacuum
        .set_default("vacuum.bronze", 30)?
        .set_default("vacuum.silver", 30)?
        .set_default("vacuum.gold", 0)?
        // schedule
        .set_default("schedule.silver", 2)?
        .set_default("schedule.gold", 0)?
        .set_default("schedule.vacuum", 30)?
        // request
        .set_default("request.channel_size", 100)?
        .set_default("request.consumer_batch_size", 10)?
        // ps
        .set_default("ps.channel_size", 500)?
        .set_default("ps.producer_frequency", 1)?
        .set_default("ps.consumer_batch_size", 50)?
        // lsof
        .set_default("lsof.channel_size", 1000)?
        .set_default("lsof.producer_frequency", 10)?
        .set_default("lsof.consumer_batch_size", 150)?
        // network
        .set_default("network.channel_size", 100)?
        .set_default("network.consumer_batch_size", 5)?;

    let config_file = Path::new(CONFIG_FILE_PATH);
    if config_file.exists() {
        config = config.add_source(config::File::with_name(CONFIG_FILE_PATH));
    }

    let config = config.build()?;

    Ok(config.try_deserialize()?)
}
