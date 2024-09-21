use crate::capture::Capture;
use crate::error::Error;
use futures::future::join_all;
use pcap::Device;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use tracing::{error, info};

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
                    error!("{}: {}", &device_name, e);
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
    let devices: Vec<Device> = Device::list()?
        .into_iter()
        .filter(|device| device.flags.is_up() && device.flags.is_running())
        .collect();

    info!("network producer scan {} devices", devices.len());

    let tasks: Vec<_> = devices
        .into_iter()
        .map(|device| {
            let sender = sender.clone();
            let stop_flag = Arc::clone(stop_flag);
            tokio::spawn(async move { read_device(device, sender, stop_flag).await })
        })
        .collect();
    let results = join_all(tasks).await;
    for result in results {
        result??
    }

    info!("all interface producer stop gracefully");

    Ok(())
}

/*
#[cfg(test)]
mod tests {

    use crate::capture::Capture;
    use crate::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::join;
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use tokio::time::sleep;

    #[tokio::test]
    async fn producer_integration_test() {
        let (sender, mut receiver): (Sender<Capture>, Receiver<Capture>) = channel(256);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = Arc::clone(&stop_flag);

        let producer_task = tokio::spawn(async move {
            producer(sender, stop_flag_clone).await.unwrap();
        });

        let stop_task = tokio::spawn(async move {
            sleep(Duration::from_secs(5)).await;
            stop_flag.store(true, Ordering::Release);
        });

        let mut captures: Vec<Capture> = vec![];
        while let Some(capture) = receiver.recv().await {
            captures.push(capture);
        }

        let (producer_task_result, stop_task_result) = join!(producer_task, stop_task);
        producer_task_result.unwrap();
        stop_task_result.unwrap();

        //assert!(captures.len() > 1);
        //assert!(captures.first().unwrap().data_link.is_some())
    }
}
*/
