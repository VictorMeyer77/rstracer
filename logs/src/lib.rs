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

// TODO: add macos log files
const MACOS_LOG_FILES: &[&str] = &[];

// TODO: add windows log files
const WINDOWS_LOG_FILES: &[&str] = &[];

const TEST_LOG_FILES: &[&str] = &["file0.txt", "file1.txt"];

// TODO: add schema to log for parsing
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
            "windows" => Ok(WINDOWS_LOG_FILES),
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
    // Stop flag is checked after each "next_line".
    // To stop properly, a new line should be parsed after set stop_flag to false.
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
    use std::fs::File;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use std::{env, fs};
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncWriteExt;
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use tokio::time::sleep;
    use tokio::{join, task};

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
        File::create(TEST_LOG_FILES[0]).unwrap();
        let (sender, mut receiver): (Sender<Log>, Receiver<Log>) = channel(100);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let producer_task =
            task::spawn(async move { producer(sender, stop_flag_clone).await.unwrap() });
        let write_file_task = task::spawn(async move {
            let mut file = OpenOptions::new()
                .append(true)
                .open(TEST_LOG_FILES[0])
                .await
                .unwrap();
            file.write_all(b"row 0\nrow 1\nrow 2\nrow 3\n")
                .await
                .unwrap();
            sleep(Duration::from_secs(1)).await;
            stop_flag.store(true, Ordering::Release);
            sleep(Duration::from_secs(1)).await;
            file.write_all(b"row 4\n").await.unwrap();
        });

        let mut logs: Vec<Log> = vec![];
        while let Some(log) = receiver.recv().await {
            logs.push(log);
        }

        let (producer_task_result, write_file_task_result) = join!(producer_task, write_file_task);
        producer_task_result.unwrap();
        write_file_task_result.unwrap();

        assert_eq!(logs.len(), 5);
        assert_eq!(
            logs.last().unwrap().file,
            Path::new(TEST_LOG_FILES[0])
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        );
        assert_eq!(logs.last().unwrap().row, "row 4");

        fs::remove_file(TEST_LOG_FILES[0]).unwrap();
    }
}
