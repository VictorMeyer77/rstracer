use crate::ps::error::Error;
use crate::ps::unix::Unix;
use std::env::consts;
use std::process::Output;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

pub mod error;
pub mod unix;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Process {
    pub pid: u32,        // Process ID
    pub ppid: u32,       // Parent Process ID
    pub uid: u32,        // User ID of the process owner
    pub lstart: i64,     // Exact date and time when the process started
    pub pcpu: f32,       // CPU usage percentage
    pub pmem: f32,       // Memory usage percentage
    pub status: String,  // Process status
    pub command: String, // Command with all its arguments
    pub created_at: i64, // Timestamp command execution
}

pub trait Ps {
    fn os_command() -> Result<Output, Error>;
    fn parse_output(output: &str) -> Result<Vec<Process>, Error> {
        let mut processes: Vec<Process> = vec![];
        for row in output.lines().skip(1) {
            if let Ok(process) = Self::parse_row(row) {
                processes.push(process)
            } else {
                return Err(Error::ParseProcess {
                    process: row.to_string(),
                });
            }
        }
        Ok(processes)
    }
    fn parse_row(row: &str) -> Result<Process, Error>;
    fn parse_date(date_chunks: &[&str]) -> Result<i64, Error>;
    fn exec() -> Result<Vec<Process>, Error> {
        let output = Self::os_command()?;
        Self::parse_output(&String::from_utf8_lossy(&output.stdout))
    }
}

pub fn ps() -> Result<Vec<Process>, Error> {
    if ["linux", "macos"].contains(&consts::OS) {
        Unix::exec()
    } else {
        Err(Error::Unimplemented {
            os: consts::OS.to_string(),
            arch: consts::ARCH.to_string(),
        })
    }
}

pub async fn producer(
    sender: Sender<Process>,
    stop_flag: Arc<AtomicBool>,
    frequency_secs: u64,
) -> Result<(), Error> {
    while !stop_flag.load(Ordering::Relaxed) {
        for file in ps()? {
            if let Err(e) = sender.send(file).await {
                return Err(Error::Channel(Box::new(e)));
            }
        }
        sleep(Duration::from_secs(frequency_secs)).await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ps::{producer, Process};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tokio::join;
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_producer_integration() {
        let (sender, mut receiver): (Sender<Process>, Receiver<Process>) = channel(256);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let producer_task = tokio::spawn(async move {
            producer(sender, stop_flag_clone, 1).await.unwrap();
        });

        let stop_task = tokio::spawn(async move {
            sleep(Duration::from_secs(1)).await;
            stop_flag.store(true, Ordering::Release);
        });

        let mut processes: Vec<Process> = vec![];
        while let Some(process) = receiver.recv().await {
            processes.push(process);
        }

        let (producer_task_result, stop_task_result) = join!(producer_task, stop_task);
        producer_task_result.unwrap();
        stop_task_result.unwrap();

        assert!(processes.len() > 100);
    }
}
