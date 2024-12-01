use network::capture::Capture;
use network::producer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{join, signal};
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_max_level(Level::INFO)
        .init();

    let (sender, mut receiver): (Sender<Capture>, Receiver<Capture>) = channel(256);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    let producer_task = tokio::spawn(async move { producer(sender, &stop_flag_clone).await });

    let stop_task = tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        stop_flag.store(true, Ordering::Release);
    });

    while let Some(capture) = receiver.recv().await {
        println!("{:?}", capture);
    }

    let (producer_task_result, stop_task_result) = join!(producer_task, stop_task);
    producer_task_result.unwrap().unwrap();
    stop_task_result.unwrap();
}
