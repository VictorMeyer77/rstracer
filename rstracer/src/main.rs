use rstracer::run;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::{join, signal};

#[tokio::main]
async fn main() {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    let main_task = tokio::spawn(async move { run(stop_flag_clone).await });

    let stop_task = tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        stop_flag.store(true, Ordering::Release);
    });

    let (main_task_result, stop_task_result) = join!(main_task, stop_task);
    main_task_result.unwrap().unwrap();
    stop_task_result.unwrap();
}
