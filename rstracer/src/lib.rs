use crate::config::read_config;
use crate::pipeline::database::{copy_layer, execute_request, get_schema};
use crate::pipeline::error::Error;
use crate::pipeline::stage::schema::{create_schema_request, Schema};
use crate::pipeline::{
    execute_request_task, execute_schedule_request_task, network_capture_sink_task,
    open_file_sink_task, process_task,
};
use network::capture::Capture;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::join;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::error;

pub mod config;
pub mod pipeline;

pub async fn start(stop_flag: Arc<AtomicBool>) -> Result<(), Error> {
    let config = read_config()?;
    execute_request(&create_schema_request(&config.disk_file_path))?;

    let schema = get_schema()?;
    copy_layer(&schema, "disk", "memory", config.load_layer.to_list())?;

    let (sender_request, receiver_request): (Sender<String>, Receiver<String>) =
        channel(config.request.channel_size);
    let (sender_open_file, receiver_open_file): (
        Sender<lsof::lsof::OpenFile>,
        Receiver<lsof::lsof::OpenFile>,
    ) = channel(config.lsof.channel_size);
    let (sender_capture, receiver_capture): (Sender<Capture>, Receiver<Capture>) =
        channel(config.network.channel_size);

    let execute_schedule_request_task = start_schedule_request_task(&config, &schema, &stop_flag);
    let execute_request_task = start_execute_request_task(&config, receiver_request, &stop_flag);
    let process_task = start_process_task(&config, &sender_request, &stop_flag);
    let open_file_source_task = start_open_file_source_task(&config, sender_open_file, &stop_flag);
    let open_file_sink_task =
        start_open_file_sink_task(&config, receiver_open_file, &sender_request, &stop_flag);
    let network_capture_source_task = start_network_capture_source_task(sender_capture, &stop_flag);
    let network_capture_sink_task =
        start_network_capture_sink_task(&config, receiver_capture, &sender_request, &stop_flag);

    let (
        execute_schedule_request_result,
        execute_request_result,
        process_result,
        open_file_source_result,
        open_file_sink_result,
        network_capture_source_result,
        network_capture_sink_result,
    ) = join!(
        execute_schedule_request_task,
        execute_request_task,
        process_task,
        open_file_source_task,
        open_file_sink_task,
        network_capture_source_task,
        network_capture_sink_task,
    );

    copy_layer(&schema, "memory", "disk", config.persist_layer.to_list())?;

    execute_schedule_request_result?;
    execute_request_result?;
    process_result?;
    open_file_source_result?;
    open_file_sink_result?;
    network_capture_source_result?;
    network_capture_sink_result?;

    Ok(())
}

fn start_execute_request_task(
    config: &config::Config,
    receiver_request: Receiver<String>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.request.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) = execute_request_task(&config_clone, receiver_request, stop_flag_read).await
        {
            stop_flag_write.store(true, Ordering::Release);
            error!("{}", e);
        }
    })
}

fn start_schedule_request_task(
    config: &config::Config,
    schema: &Schema,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.clone();
    let schema_clone = schema.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) =
            execute_schedule_request_task(&config_clone, schema_clone, stop_flag_read).await
        {
            stop_flag_write.store(true, Ordering::Release);
            error!("{}", e);
        }
    })
}

fn start_process_task(
    config: &config::Config,
    sender_request: &Sender<String>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.ps.clone();
    let sender_clone = sender_request.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) = process_task(&config_clone, sender_clone, stop_flag_read).await {
            stop_flag_write.store(true, Ordering::Release);
            error!("{}", e);
        }
    })
}

fn start_open_file_source_task(
    config: &config::Config,
    sender_open_file: Sender<lsof::lsof::OpenFile>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.lsof.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) = lsof::lsof::producer(
            sender_open_file,
            stop_flag_read,
            config_clone.producer_frequency.unwrap(),
        )
        .await
        {
            stop_flag_write.store(true, Ordering::Release);
            error!("{}", e);
        }
    })
}

fn start_open_file_sink_task(
    config: &config::Config,
    receiver: Receiver<lsof::lsof::OpenFile>,
    sender_request: &Sender<String>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.lsof.clone();
    let sender_clone = sender_request.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) =
            open_file_sink_task(&config_clone, receiver, sender_clone, stop_flag_read).await
        {
            stop_flag_write.store(true, Ordering::Release);
            error!("{}", e);
        }
    })
}

fn start_network_capture_source_task(
    sender_capture: Sender<Capture>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) = network::producer(sender_capture, &stop_flag_read).await {
            stop_flag_write.store(true, Ordering::Release);
            error!("{}", e);
        }
    })
}

fn start_network_capture_sink_task(
    config: &config::Config,
    receiver: Receiver<Capture>,
    sender_request: &Sender<String>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.network.clone();
    let sender_clone = sender_request.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) =
            network_capture_sink_task(&config_clone, receiver, sender_clone, stop_flag_read).await
        {
            stop_flag_write.store(true, Ordering::Release);
            error!("{}", e);
        }
    })
}
