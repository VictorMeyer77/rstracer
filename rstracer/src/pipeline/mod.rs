use crate::config::{ChannelConfig, Config};
use crate::pipeline::database::execute_request;
use crate::pipeline::error::Error;
use crate::pipeline::stage::bronze::{Bronze, BronzeBatch};
use crate::pipeline::stage::schema::Schema;
use crate::pipeline::stage::silver::silver_request;
use crate::pipeline::stage::vacuum::vacuum_request;
use chrono::Local;
use lsof::lsof::OpenFile;
use network::capture::Capture;
use ps::ps::Process;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

pub mod database;
pub mod error;
pub mod stage;

const TIMEOUT_MS: u64 = 1000;

pub async fn execute_request_task(
    config: &ChannelConfig,
    receiver_request: Receiver<String>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Error> {
    let mut receiver_request = receiver_request;

    while !stop_flag.load(Ordering::Relaxed) {
        let mut request_buffer: Vec<String> = Vec::with_capacity(config.consumer_batch_size);

        info!(
            "sql request receiver contains {} / {} elements",
            receiver_request.len(),
            config.channel_size
        );

        match timeout(
            Duration::from_millis(TIMEOUT_MS),
            receiver_request.recv_many(&mut request_buffer, config.consumer_batch_size),
        )
        .await
        {
            Ok(_) => {
                info!(
                    "batch read {} / {} sql",
                    request_buffer.len(),
                    config.consumer_batch_size
                );
                let start = Local::now().timestamp_millis();
                execute_request(&request_buffer.join(" "))?;
                let duration = Local::now().timestamp_millis() - start;
                info!(
                    "batch execute sql requests in {} s",
                    duration as f32 / 1000.0
                );
            }
            Err(_) => {
                info!("sql request timeout triggered")
            }
        }
    }

    info!("consumer stop gracefully");

    Ok(())
}

pub async fn schedule_request_task(
    config: &Config,
    schema: Schema,
    sender_request: Sender<String>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Error> {
    let mut tasks: HashMap<(&str, String, u64), i64> = HashMap::new();
    tasks.insert(
        ("silver", silver_request(), config.schedule.silver),
        Local::now().timestamp(),
    );
    tasks.insert(
        (
            "vacuum",
            vacuum_request(config.vacuum.clone(), schema),
            config.schedule.vacuum,
        ),
        Local::now().timestamp(),
    );

    while !stop_flag.load(Ordering::Relaxed) {
        for (task, executed_at) in tasks.iter_mut() {
            if Local::now().timestamp() > *executed_at + task.2 as i64 {
                if let Err(e) = sender_request.send(task.1.clone()).await {
                    warn!("{}", e);
                    stop_flag.store(true, Ordering::Release);
                } else {
                    *executed_at = Local::now().timestamp();
                    info!("{} sql request sent", task.0);
                }
            }
        }

        sleep(Duration::from_millis(1000)).await;
    }

    info!("consumer stop gracefully");
    Ok(())
}

pub async fn process_sink_task(
    config: &ChannelConfig,
    receiver_process: Receiver<Process>,
    sender_request: Sender<String>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Error> {
    let mut receiver_process = receiver_process;

    while !stop_flag.load(Ordering::Relaxed) {
        let mut process_buffer: Vec<Process> = Vec::with_capacity(config.consumer_batch_size);

        info!(
            "process receiver contains {} elements",
            receiver_process.len()
        );

        match timeout(
            Duration::from_millis(TIMEOUT_MS),
            receiver_process.recv_many(&mut process_buffer, config.consumer_batch_size),
        )
        .await
        {
            Ok(_) => {
                let length = process_buffer.len();
                info!(
                    "process batch read {} / {}",
                    length, config.consumer_batch_size
                );
                let start = Local::now().timestamp_millis();

                let values: Vec<String> = process_buffer
                    .iter()
                    .map(|process| process.to_insert_value())
                    .collect();
                let request = if length == 0 {
                    "".to_string()
                } else {
                    format!("{} {};", Process::get_insert_header(), values.join(","))
                };
                if let Err(e) = sender_request.send(request).await {
                    warn!("{}", e);
                    stop_flag.store(true, Ordering::Release);
                } else {
                    let duration = Local::now().timestamp_millis() - start;
                    info!(
                        "sent {} process sql in {} s",
                        length,
                        duration as f32 / 1000.0
                    );
                }
            }
            Err(_) => {
                info!("process timeout triggered")
            }
        }
    }

    info!("process producer stop gracefully");

    Ok(())
}

pub async fn open_file_sink_task(
    config: &ChannelConfig,
    receiver_open_file: Receiver<OpenFile>,
    sender_request: Sender<String>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Error> {
    let mut receiver_open_file = receiver_open_file;

    while !stop_flag.load(Ordering::Relaxed) {
        let mut open_file_buffer: Vec<OpenFile> = Vec::with_capacity(config.consumer_batch_size);

        info!(
            "open file receiver contains {} elements",
            receiver_open_file.len()
        );

        match timeout(
            Duration::from_millis(TIMEOUT_MS),
            receiver_open_file.recv_many(&mut open_file_buffer, config.consumer_batch_size),
        )
        .await
        {
            Ok(_) => {
                let length = open_file_buffer.len();
                info!(
                    "open file batch read {} / {}",
                    length, config.consumer_batch_size
                );
                let start = Local::now().timestamp_millis();

                let values: Vec<String> = open_file_buffer
                    .iter()
                    .map(|file| file.to_insert_value())
                    .collect();
                let request = if length == 0 {
                    "".to_string()
                } else {
                    format!("{} {};", OpenFile::get_insert_header(), values.join(","))
                };
                if let Err(e) = sender_request.send(request).await {
                    warn!("{}", e);
                    stop_flag.store(true, Ordering::Release);
                } else {
                    let duration = Local::now().timestamp_millis() - start;
                    info!(
                        "sent {} open file sql in {} s",
                        length,
                        duration as f32 / 1000.0
                    );
                }
            }
            Err(_) => {
                info!("open file timeout triggered")
            }
        }
    }

    info!("process producer stop gracefully");

    Ok(())
}

pub async fn network_capture_sink_task(
    config: &ChannelConfig,
    receiver_capture: Receiver<Capture>,
    sender_request: Sender<String>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Error> {
    let mut receiver_capture = receiver_capture;

    while !stop_flag.load(Ordering::Relaxed) {
        let mut capture_buffer: Vec<Capture> = Vec::with_capacity(config.consumer_batch_size);

        info!(
            "network capture receiver contains {} elements",
            receiver_capture.len()
        );

        match timeout(
            Duration::from_millis(TIMEOUT_MS),
            receiver_capture.recv_many(&mut capture_buffer, config.consumer_batch_size),
        )
        .await
        {
            Ok(_) => {
                let length = capture_buffer.len();
                info!(
                    "network capture batch read {} / {}",
                    length, config.consumer_batch_size
                );
                let start = Local::now().timestamp_millis();

                let values: Vec<String> = capture_buffer.iter().map(|file| file.to_sql()).collect();

                let request = if length == 0 {
                    "".to_string()
                } else {
                    values.join(" ")
                };

                if let Err(e) = sender_request.send(request).await {
                    warn!("{}", e);
                    stop_flag.store(true, Ordering::Release);
                } else {
                    let duration = Local::now().timestamp_millis() - start;
                    info!(
                        "sent {} network capture sql in {} s",
                        length,
                        duration as f32 / 1000.0
                    );
                }
            }
            Err(_) => {
                info!("network capture timeout triggered")
            }
        }
    }

    Ok(())
}
