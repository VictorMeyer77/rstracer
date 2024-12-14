use ps::{producer, Process};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{join, signal};

const FREQUENCY: f32 = 1.0;

#[tokio::main]
async fn main() {
    let (sender, mut receiver): (Sender<Process>, Receiver<Process>) = channel(256);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();
    let producer_task =
        tokio::spawn(async move { producer(sender, &stop_flag_clone, FREQUENCY).await });

    let stop_task = tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        stop_flag.store(true, Ordering::Release);
    });

    while let Some(process) = receiver.recv().await {
        println!("{:?}", process);
    }

    let (producer_task_result, stop_task_result) = join!(producer_task, stop_task);
    producer_task_result.unwrap().unwrap();
    stop_task_result.unwrap();
}
