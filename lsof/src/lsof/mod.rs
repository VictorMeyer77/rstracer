use crate::lsof::error::Error;
use crate::lsof::unix::Unix;
use chrono::Local;
use std::env::consts;
use std::process::Output;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

pub mod error;
pub mod unix;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct OpenFile {
    pub command: String, // Command
    pub pid: u32,        // Process ID
    pub uid: u32,        // User ID
    pub fd: String,      // File Descriptor
    pub _type: String,   // Column type
    pub device: String,  // Device
    pub size: u32,       // Size
    pub node: String,    // Node
    pub name: String,    // Name
    pub created_at: i64, // Timestamp command execution
}

impl OpenFile {
    pub fn new(pid: u32, uid: u32, command: &str) -> Self {
        OpenFile {
            command: command.to_string(),
            pid,
            uid,
            fd: "".to_string(),
            _type: "".to_string(),
            device: "".to_string(),
            size: 0,
            node: "".to_string(),
            name: "".to_string(),
            created_at: Local::now().timestamp_millis(),
        }
    }
}

pub trait Lsof {
    fn os_command() -> Result<Output, Error>;
    fn parse_output(output: &str) -> Result<Vec<OpenFile>, Error>;
    fn exec() -> Result<Vec<OpenFile>, Error> {
        let output = Self::os_command()?;
        Self::parse_output(&String::from_utf8_lossy(&output.stdout))
    }
}

pub fn lsof() -> Result<Vec<OpenFile>, Error> {
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
    sender: Sender<OpenFile>,
    stop_flag: Arc<AtomicBool>,
    frequency_secs: u64,
) -> Result<(), Error> {
    while !stop_flag.load(Ordering::Relaxed) {
        for file in lsof()? {
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
    use crate::lsof::{producer, OpenFile};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tokio::join;
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_producer_integration() {
        let (sender, mut receiver): (Sender<OpenFile>, Receiver<OpenFile>) = channel(256);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let producer_task = tokio::spawn(async move {
            producer(sender, stop_flag_clone, 1).await.unwrap();
        });

        let stop_task = tokio::spawn(async move {
            sleep(Duration::from_secs(3)).await;
            stop_flag.store(true, Ordering::Release);
        });

        let mut files: Vec<OpenFile> = vec![];
        while let Some(file) = receiver.recv().await {
            files.push(file);
        }

        let (producer_task_result, stop_task_result) = join!(producer_task, stop_task);
        producer_task_result.unwrap();
        stop_task_result.unwrap();

        assert!(files.len() > 1000);
        assert_eq!(files.last().unwrap().command, "lsof");
    }
}
