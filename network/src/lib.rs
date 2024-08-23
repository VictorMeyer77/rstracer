use crate::capture::Capture;
use crate::error::{handle_error, Error};
use pcap::Device;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use tracing::info;

pub mod capture;
pub mod error;

pub async fn producer(sender: Sender<Capture>, stop_flag: Arc<AtomicBool>) -> Result<(), Error> {
    let device = Device::lookup()?.unwrap();
    info!("use device {}", device.name);

    let capture = pcap::Capture::from_device(device)?
        .promisc(true)
        .timeout(100)
        .open()?;

    let mut capture = capture.setnonblock()?;

    while !stop_flag.load(Ordering::Relaxed) {
        match capture.next_packet() {
            Ok(packet) => match Capture::parse(packet.data) {
                Ok(capture) => {
                    if let Err(e) = sender.send(capture).await {
                        handle_error(Error::Channel(Box::new(e)));
                    }
                }
                Err(e) => handle_error(e),
            },
            Err(e) => handle_error(Error::Pcap(e)),
        }
        sleep(Duration::from_millis(100)).await;
    }
    info!("producer stop gracefully");
    Ok(())
}

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

        assert!(captures.len() > 1);
        assert!(captures.first().unwrap().data_link.is_some())
    }
}
