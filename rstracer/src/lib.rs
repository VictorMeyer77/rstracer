use crate::config::read_config;
use crate::pipeline::database::{copy_layer, execute_request, get_schema};
use crate::pipeline::error::Error;
use crate::pipeline::stage::schema::{create_schema_request, Schema};
use crate::pipeline::{
    execute_request_task, open_file_sink_task, process_sink_task, schedule_request_task,
};
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
    let (sender_process, receiver_process): (Sender<ps::ps::Process>, Receiver<ps::ps::Process>) =
        channel(config.ps.channel_size);
    let (sender_open_file, receiver_open_file): (
        Sender<lsof::lsof::OpenFile>,
        Receiver<lsof::lsof::OpenFile>,
    ) = channel(config.lsof.channel_size);

    let schedule_request_task =
        start_schedule_request_task(&config, &schema, &sender_request, &stop_flag);
    let execute_request_task = start_execute_request_task(&config, receiver_request, &stop_flag);
    let process_source_task = start_process_source_task(&config, sender_process, &stop_flag);
    let process_sink_task =
        start_process_sink_task(&config, receiver_process, &sender_request, &stop_flag);
    let open_file_source_task = start_open_file_source_task(&config, sender_open_file, &stop_flag);
    let open_file_sink_task =
        start_open_file_sink_task(&config, receiver_open_file, &sender_request, &stop_flag);

    let (
        schedule_request_result,
        execute_request_result,
        process_source_result,
        process_sink_result,
        open_file_source_result,
        open_file_sink_result,
    ) = join!(
        schedule_request_task,
        execute_request_task,
        process_source_task,
        process_sink_task,
        open_file_source_task,
        open_file_sink_task
    );

    copy_layer(&schema, "memory", "disk", config.persist_layer.to_list())?;

    schedule_request_result?;
    execute_request_result?;
    process_source_result?;
    process_sink_result?;
    open_file_source_result?;
    open_file_sink_result?;

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
    sender_request: &Sender<String>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.clone();
    let schema_clone = schema.clone();
    let sender_request_clone = sender_request.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) = schedule_request_task(
            &config_clone,
            schema_clone,
            sender_request_clone,
            stop_flag_read,
        )
        .await
        {
            stop_flag_write.store(true, Ordering::Release);
            error!("{}", e);
        }
    })
}

fn start_process_source_task(
    config: &config::Config,
    sender_process: Sender<ps::ps::Process>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.ps.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) = ps::ps::producer(
            sender_process,
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

fn start_process_sink_task(
    config: &config::Config,
    receiver: Receiver<ps::ps::Process>,
    sender_request: &Sender<String>,
    stop_flag: &Arc<AtomicBool>,
) -> JoinHandle<()> {
    let config_clone = config.ps.clone();
    let sender_clone = sender_request.clone();
    let stop_flag_read = stop_flag.clone();
    let stop_flag_write = stop_flag.clone();
    tokio::spawn(async move {
        if let Err(e) =
            process_sink_task(&config_clone, receiver, sender_clone, stop_flag_read).await
        {
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
