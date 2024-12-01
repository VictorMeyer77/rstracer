use crate::capture::Capture;
use crate::error::Error;
use pcap::Device;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub mod capture;
pub mod error;

pub async fn read_device(
    device: Device,
    sender: Sender<Capture>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), Error> {
    let device_name = device.name.clone();
    info!("read device {}", device_name);

    let capture = pcap::Capture::from_device(device.clone())?
        .timeout(100)
        .open()?;

    let mut capture = capture.setnonblock()?;

    while !stop_flag.load(Ordering::Relaxed) {
        match capture.next_packet() {
            Ok(packet) => match Capture::parse(packet.data, &device) {
                Ok(capture) => {
                    if let Err(e) = sender.send(capture).await {
                        error!("{}: {}", &device_name, e);
                        break;
                    }
                }
                Err(e) => {
                    error!("{}: {}", &device_name, e);
                    break;
                }
            },
            Err(e) => match e {
                pcap::Error::TimeoutExpired => {}
                _ => {
                    warn!("{}: {}", &device_name, e);
                    break;
                }
            },
        }
        sleep(Duration::from_millis(10)).await;
    }
    info!("producer on device {} stop gracefully", device_name);
    Ok(())
}

pub async fn producer(sender: Sender<Capture>, stop_flag: &Arc<AtomicBool>) -> Result<(), Error> {
    let mut tasks: HashMap<String, JoinHandle<Result<(), Error>>> = HashMap::new();

    while !stop_flag.load(Ordering::Relaxed) {
        let devices: Vec<Device> = Device::list()?
            .into_iter()
            .filter(|device| device.flags.is_up() && device.flags.is_running())
            .collect();

        for device in devices {
            let device_name = device.name.clone();
            let mut is_running = false;
            if let Some(task) = tasks.get(&device_name) {
                is_running = !task.is_finished();
            }
            if !is_running {
                let sender = sender.clone();
                let stop_flag = Arc::clone(stop_flag);
                let task =
                    tokio::spawn(async move { read_device(device, sender, stop_flag).await });
                tasks.insert(device_name, task);
            }
        }

        sleep(Duration::from_secs(10)).await;
    }

    join_device_tasks(&tasks)?;

    Ok(())
}

fn join_device_tasks(tasks: &HashMap<String, JoinHandle<Result<(), Error>>>) -> Result<(), Error> {
    for task in tasks.values() {
        if !task.is_finished() {
            thread::sleep(Duration::from_millis(10));
            join_device_tasks(tasks)?;
        }
    }
    info!("all interface producer stop gracefully");
    Ok(())
}
