use crate::config::{ChannelConfig, Config};
use crate::pipeline::database::execute_request;
use crate::pipeline::error::Error;
use crate::pipeline::stage::bronze::{concat_requests, create_insert_batch_request, Bronze};
use crate::pipeline::stage::{file, gold, silver, vacuum};
use chrono::Local;
use lsof::lsof::{lsof, OpenFile};
use network::capture::Capture;
use ps::ps::{ps, Process};
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
    in_memory: bool,
) -> Result<(), Error> {
    let mut receiver_request = receiver_request;

    while !stop_flag.load(Ordering::Relaxed) {
        let mut request_buffer: Vec<String> = Vec::with_capacity(config.consumer_batch_size);

        info!(
            "sql request receiver contains {} / {} elements",
            receiver_request.len(),
            config.channel_size.unwrap()
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
                execute_request(&request_buffer.join(""), in_memory)?;
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

fn get_schedule_request_task(config: &Config) -> Result<HashMap<(&str, String, u64), i64>, Error> {
    let mut tasks: HashMap<(&str, String, u64), i64> = HashMap::new();
    tasks.insert(
        ("silver", silver::request(), config.schedule.silver),
        Local::now().timestamp(),
    );
    tasks.insert(
        ("gold", gold::request(), config.schedule.gold),
        Local::now().timestamp() + 1,
    );
    tasks.insert(
        (
            "vacuum",
            vacuum::request(config.vacuum.clone()),
            config.schedule.vacuum,
        ),
        Local::now().timestamp(),
    );
    tasks.insert(("file", file::request()?, config.schedule.file), 0);
    Ok(tasks)
}

pub async fn execute_schedule_request_task(
    config: &Config,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Error> {
    let mut tasks = get_schedule_request_task(config)?;
    while !stop_flag.load(Ordering::Relaxed) {
        for (task, executed_at) in tasks.iter_mut() {
            let now = Local::now().timestamp();
            if now > *executed_at + task.2 as i64 {
                let start = Local::now().timestamp_millis();
                execute_request(&task.1, config.in_memory)?;
                *executed_at = now;
                let duration = Local::now().timestamp_millis() - start;

                info!(
                    "{} requests executed in {} s",
                    task.0,
                    duration as f32 / 1000.0
                );
            }
        }

        sleep(Duration::from_millis(10)).await;
    }

    info!("consumer stop gracefully");
    Ok(())
}

pub async fn process_task(
    config: &ChannelConfig,
    sender_request: Sender<String>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Error> {
    let frequency = config.producer_frequency.unwrap();

    while !stop_flag.load(Ordering::Relaxed) {
        let start = Local::now().timestamp_millis();
        let processes = ps()?;
        let length = processes.len();

        let batches: Vec<Vec<Process>> = processes
            .chunks(config.consumer_batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        for batch in batches {
            if let Err(e) = sender_request
                .send(create_insert_batch_request(batch))
                .await
            {
                warn!("{}", e);
                stop_flag.store(true, Ordering::Release);
            }
        }

        let duration = Local::now().timestamp_millis() - start;

        if duration > (frequency * 1000) as i64 {
            warn!(
                "sending rows is longer than the frequency. {} bronze processes sent in {} s",
                length,
                duration as f32 / 1000.0
            );
        } else {
            info!(
                "sent bronze sql request with {} processes in {} s",
                length,
                duration as f32 / 1000.0
            );
            sleep(Duration::from_millis(frequency * 1000 - duration as u64)).await;
        }
    }

    info!("process producer stop gracefully");

    Ok(())
}

pub async fn open_file_task(
    config: &ChannelConfig,
    sender_request: Sender<String>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Error> {
    let frequency = config.producer_frequency.unwrap();

    while !stop_flag.load(Ordering::Relaxed) {
        let start = Local::now().timestamp_millis();
        let open_files = lsof()?;
        let length = open_files.len();

        let batches: Vec<Vec<OpenFile>> = open_files
            .chunks(config.consumer_batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        for batch in batches {
            if let Err(e) = sender_request
                .send(create_insert_batch_request(batch))
                .await
            {
                warn!("{}", e);
                stop_flag.store(true, Ordering::Release);
            }
        }

        let duration = Local::now().timestamp_millis() - start;

        if duration > (frequency * 1000) as i64 {
            warn!(
                "sending rows is longer than the frequency. {} bronze open files sent in {} s",
                length,
                duration as f32 / 1000.0
            );
        } else {
            info!(
                "sent bronze sql request with {} open files in {} s",
                length,
                duration as f32 / 1000.0
            );
            sleep(Duration::from_millis(frequency * 1000 - duration as u64)).await;
        }
    }

    info!("open file producer stop gracefully");

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

                let values: Vec<String> = capture_buffer
                    .iter()
                    .map(|file| file.to_insert_sql(None))
                    .collect();

                let request = concat_requests(values, config.consumer_batch_size).join(";");

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

        sleep(Duration::from_secs(config.producer_frequency.unwrap())).await;
    }

    Ok(())
}
