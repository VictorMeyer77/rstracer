use crate::error::Error;
use chrono::Local;
use linemux::MuxedLines;
use std::env;
use std::env::consts;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub mod error;

const LINUX_LOG_FILES: &[&str] = &[
    "/var/log/alternatives.log",
    "/var/log/auth.log",
    "/var/log/user.log",
    "/var/log/syslog",
    "/var/log/dmesg",
    "/var/log/boot.log",
    "/var/log/cron",
    "/var/log/daemon.log",
    "/var/log/apt/term.log",
    "/var/log/apt/history.log",
    "/var/log/dpkg.log",
    "/var/log/faillog",
    "/var/log/kern.log",
];

const MACOS_LOG_FILES: &[&str] = &["TODO"];

const WINDOWS_LOG_FILES: &[&str] = &["TODO"];

const TEST_LOG_FILES: &[&str] = &["file0.txt", "file1.txt"];

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Log {
    pub file: String,
    pub date: i64,
    pub row: String,
}

async fn set_log_files(lines: &mut MuxedLines) -> Result<(), Error> {
    let files = if env::var("RUST_TEST_LOGS").is_ok() {
        Ok(TEST_LOG_FILES)
    } else {
        match consts::OS {
            "linux" => Ok(LINUX_LOG_FILES),
            "macos" => Ok(MACOS_LOG_FILES),
            _ => Err(Error::Unimplemented {
                os: consts::OS.to_string(),
            }),
        }
    };
    for file in files? {
        lines.add_file(file).await?;
    }
    Ok(())
}

pub async fn producer(sender: Sender<Log>, stop_flag: Arc<AtomicBool>) -> Result<(), Error> {
    let mut lines = MuxedLines::new()?;
    set_log_files(&mut lines).await?;
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
    use crate::{producer, set_log_files, Log, TEST_LOG_FILES};
    use linemux::MuxedLines;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use std::{env, fs};
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncWriteExt;
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use tokio::task;
    use tokio::time::sleep;

    fn set_test_env() {
        env::set_var("RUST_TEST_LOGS", "true");
    }

    #[tokio::test]
    async fn set_log_files_should_init_muxed_lines() {
        set_test_env();
        let mut lines = MuxedLines::new().unwrap();
        set_log_files(&mut lines).await.unwrap();
        assert!(format!("{:?}", lines).contains(TEST_LOG_FILES[0]));
        assert!(format!("{:?}", lines).contains(TEST_LOG_FILES[1]));
    }

    #[tokio::test]
    async fn producer_integration_test() {
        set_test_env();
        let (sender, mut receiver): (Sender<Log>, Receiver<Log>) = channel(100);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        task::spawn(async move { producer(sender, stop_flag_clone).await });
        task::spawn(async {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(TEST_LOG_FILES[0])
                .await
                .unwrap();
            sleep(Duration::from_secs(1)).await;
            file.write_all(b"row test\n").await.unwrap();
        });
        let received = receiver.recv().await.unwrap();

        assert_eq!(
            received.file,
            Path::new(TEST_LOG_FILES[0])
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        );
        assert_eq!(received.row, "row test");

        stop_flag.store(true, Ordering::Release);
        fs::remove_file(TEST_LOG_FILES[0]).unwrap();
    }
}
