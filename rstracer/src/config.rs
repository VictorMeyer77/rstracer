use crate::pipeline::error::Error;
use config;
use serde::Deserialize;
use std::path::Path;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

const CONFIG_FILE_PATH: &str = "rstracer.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub in_memory: bool,
    pub request: ChannelConfig,
    pub ps: ChannelConfig,
    pub lsof: LsofConfig,
    pub network: ChannelConfig,
    pub vacuum: VacuumConfig,
    pub export: ExportConfig,
    pub schedule: ScheduleConfig,
    pub logger: LoggerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LsofConfig {
    pub regular: ChannelConfig,
    pub network: ChannelConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChannelConfig {
    pub channel_size: Option<usize>,
    pub producer_frequency: Option<u64>,
    pub consumer_batch_size: usize,
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
pub struct ExportConfig {
    pub directory: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScheduleConfig {
    pub silver: u64,
    pub gold: u64,
    pub vacuum: u64,
    pub file: u64,
    pub export: u64,
}

impl ScheduleConfig {
    pub fn to_list(&self) -> Vec<(String, u64)> {
        vec![
            ("silver".to_string(), self.silver),
            ("gold".to_string(), self.gold),
            ("vacuum".to_string(), self.vacuum),
            ("file".to_string(), self.file),
            ("export".to_string(), self.export),
        ]
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggerConfig {
    pub level: String,
    pub directory: Option<String>,
    pub rotation: Option<String>,
}

pub fn read_config() -> Result<Config, Error> {
    let mut config = config::Config::builder()
        .set_default("in_memory", "false")?
        // vacuum
        .set_default("vacuum.bronze", 15)?
        .set_default("vacuum.silver", 15)?
        .set_default("vacuum.gold", 600)?
        // schedule
        .set_default("schedule.silver", 10)?
        .set_default("schedule.gold", 10)?
        .set_default("schedule.vacuum", 15)?
        .set_default("schedule.file", 300)?
        .set_default("schedule.export", 60)?
        // request
        .set_default("request.channel_size", 100)?
        .set_default("request.consumer_batch_size", 20)?
        // ps
        .set_default("ps.producer_frequency", 3)?
        .set_default("ps.consumer_batch_size", 200)?
        // lsof regular
        .set_default("lsof.regular.producer_frequency", 20)?
        .set_default("lsof.regular.consumer_batch_size", 200)?
        // lsof network
        .set_default("lsof.network.producer_frequency", 3)?
        .set_default("lsof.network.consumer_batch_size", 200)?
        // network
        .set_default("network.channel_size", 500)?
        .set_default("network.producer_frequency", 1)?
        .set_default("network.consumer_batch_size", 200)?
        // export
        .set_default("export.directory", "export/")?
        .set_default("export.format", "parquet")?
        // export
        .set_default("logger.level", "INFO")?;

    let config_file = Path::new(CONFIG_FILE_PATH);
    if config_file.exists() {
        config = config.add_source(config::File::with_name(CONFIG_FILE_PATH));
    }

    let config = config.build()?;

    Ok(config.try_deserialize()?)
}

pub fn subscribe_logger(config: &LoggerConfig) {
    let level = match config.level.to_uppercase().as_str() {
        "TRACE" => Level::TRACE,
        "DEBUG" => Level::DEBUG,
        "INFO" => Level::INFO,
        "WARN" => Level::WARN,
        "ERROR" => Level::ERROR,
        level => panic!("Unknown logger level '{level}'"),
    };

    if let Some(directory) = &config.directory {
        let rotation = if let Some(rotation) = &config.rotation {
            match rotation.to_uppercase().as_str() {
                "MINUTELY" => Rotation::MINUTELY,
                "HOURLY" => Rotation::HOURLY,
                "DAILY" => Rotation::DAILY,
                rotation => panic!("Unknown log rotation '{rotation}'"),
            }
        } else {
            Rotation::HOURLY
        };
        let appender = RollingFileAppender::new(rotation, directory, "rstracer.log");
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_target(false)
            .with_writer(appender)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_target(false)
            .init();
    }
}
