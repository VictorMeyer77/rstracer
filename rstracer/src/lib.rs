use crate::config::read_config;
use crate::pipeline::database::{close_connection, execute_request};
use crate::pipeline::error::Error;
use crate::pipeline::stage::schema::create_schema_request;
use crate::pipeline::stage::{gold, silver};
use crate::pipeline::{
    execute_request_task, execute_schedule_request_task, network_capture_sink_task, open_file_task,
    process_task,
};
use lsof::lsof::FileType;
use network::capture::Capture;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::join;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::error;

pub mod config;
pub mod pipeline;

pub async fn run(stop_flag: Arc<AtomicBool>) -> Result<(), Error> {
    let config = read_config()?;
    execute_request(&create_schema_request(), config.in_memory)?;

    let (sender_request, receiver_request): (Sender<String>, Receiver<String>) =
        channel(config.request.channel_size.unwrap());
    let (sender_capture, receiver_capture): (Sender<Capture>, Receiver<Capture>) =
        channel(config.network.channel_size.unwrap());

    let execute_schedule_request_task = start_schedule_request_task(&config, &stop_flag);
    let execute_request_task = start_execute_request_task(&config, receiver_request, &stop_flag);
    let process_task = start_process_task(&config, &sender_request, &stop_flag);
    let open_file_regular_task = start_open_file_task(
        &config.lsof.regular,
        FileType::REGULAR,
        &sender_request,
        &stop_flag,
    );
    let open_file_network_task = start_open_file_task(
        &config.lsof.network,
        FileType::NETWORK,
        &sender_request,
        &stop_flag,
    );
    let network_capture_source_task = start_network_capture_source_task(sender_capture, &stop_flag);
    let network_capture_sink_task =
        start_network_capture_sink_task(&config, receiver_capture, &sender_request, &stop_flag);

    let (
        execute_schedule_request_result,
        execute_request_result,
        process_result,
        open_file_regular_result,
        open_file_network_result,
        network_capture_source_result,
        network_capture_sink_result,
    ) = join!(
        execute_schedule_request_task,
        execute_request_task,
        process_task,
        open_file_regular_task,
        open_file_network_task,
        network_capture_source_task,
        network_capture_sink_task,
    );

    execute_request(&silver::request(), config.in_memory)?;
    execute_request(&gold::request(), config.in_memory)?;
    close_connection(config.in_memory)?;

    execute_schedule_request_result?;
    execute_request_result?;
    process_result?;
    open_file_regular_result?;
    open_file_network_result?;
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
    let in_memory = config.in_memory;
    tokio::spawn(async move {
        if let Err(e) =
            execute_request_task(&config_clone, receiver_request, stop_flag_read, in_memory).await
        {
            stop_flag_write.store(true, Ordering::Release);
            error!("{}", e);
        }
    })
}

fn start_schedule_request_task(
    config: &config::Config,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) = execute_schedule_request_task(&config_clone, stop_flag_read).await {
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

fn start_open_file_task(
    config: &config::ChannelConfig,
    file_type: FileType,
    sender_request: &Sender<String>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.clone();
    let sender_clone = sender_request.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) = open_file_task(&config_clone, file_type, sender_clone, stop_flag_read).await
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
