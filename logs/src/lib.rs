use crate::error::Error;
use chrono::Local;
use linemux::MuxedLines;
use std::env;
use std::env::consts;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub mod error;

const LINUX_LOG_FILES: &'static [&str] = &["/var/log/system.log"];

const MACOS_LOG_FILES: &'static [&str] = &[
    "/var/log/system.log",
    "/Users/victormeyer/Library/Logs/JetBrains/RustRover2024.1/idea.log",
    "todo",
];

const TEST_LOG_FILE: &str = "test.txt";

#[derive(Debug)]
pub struct Log {
    pub file: String,
    pub date: i64,
    pub row: String,
}

fn get_log_files() -> Result<&'static [&'static str], Error> {
    if env::var("RUST_TEST_LOGS").is_ok() {
        Ok(&["test.txt"])
    } else {
        match consts::OS {
            "linux" => Ok(LINUX_LOG_FILES),
            "macos" => Ok(MACOS_LOG_FILES),
            _ => Err(Error::Unimplemented {
                os: consts::OS.to_string(),
            }),
        }
    }
}

pub async fn producer(sender: Sender<Log>, stop_flag: Arc<AtomicBool>) -> Result<(), Error> {
    let mut lines = MuxedLines::new()?;
    for file in get_log_files()? {
        lines.add_file(file).await?;
    }
    while !stop_flag.load(Ordering::Relaxed) {
        if let Ok(Some(line)) = lines.next_line().await {
            sender
                .send(Log {
                    file: line.source().canonicalize()?.to_str().unwrap().to_string(),
                    date: Local::now().timestamp_micros(),
                    row: line.line().to_string(),
                })
                .await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{producer, Log, TEST_LOG_FILE};
    use std::env;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncWriteExt;
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use tokio::task;
    use tokio::time::sleep;

    fn set_test_env() {
        env::set_var("RUST_TEST_LOGS", "true");
    }

    #[tokio::test]
    async fn integration_test() {
        set_test_env();
        let (sender, mut receiver): (Sender<Log>, Receiver<Log>) = channel(100);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        task::spawn(async move { producer(sender, stop_flag_clone).await });
        task::spawn(async {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(TEST_LOG_FILE)
                .await
                .unwrap();
            sleep(Duration::from_secs(1)).await;
            file.write_all(b"row test\n").await.unwrap();
        });
        let received = receiver.recv().await.unwrap();

        assert_eq!(
            received.file,
            Path::new(TEST_LOG_FILE)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        );
        assert_eq!(received.row, "row test");

        stop_flag.store(true, Ordering::Release);
    }
}
