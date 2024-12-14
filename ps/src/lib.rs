use crate::error::Error;
use chrono::{Local, NaiveDateTime};
use std::io::{BufRead, BufReader};
use std::process::{ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub mod error;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Process {
    pub pid: u32,        // Process ID
    pub ppid: u32,       // Parent Process ID
    pub uid: i16,        // User ID of the process owner
    pub lstart: i64,     // Exact date and time when the process started
    pub pcpu: f32,       // CPU usage percentage
    pub pmem: f32,       // Memory usage percentage
    pub status: String,  // Process status
    pub command: String, // Command with all its arguments
    pub created_at: i64, // Timestamp command execution
}

impl Process {
    fn from(row: &str) -> Result<Process, Error> {
        let chunks: Vec<&str> = row.split_whitespace().collect();
        Ok(Process {
            pid: chunks[0].parse()?,
            ppid: chunks[1].parse()?,
            uid: chunks[2].parse()?,
            lstart: Self::parse_unix_ctime(&chunks[3..8])?,
            pcpu: chunks[8].parse()?,
            pmem: chunks[9].parse()?,
            status: chunks[10].to_string(),
            command: chunks[11..].join(" "),
            created_at: Local::now().timestamp_millis(),
        })
    }

    fn parse_unix_ctime(date_chunks: &[&str]) -> Result<i64, Error> {
        let format = "%a %b %d %H:%M:%S %Y";
        Ok(
            NaiveDateTime::parse_from_str(date_chunks.join(" ").as_str(), format)?
                .and_local_timezone(Local)
                .unwrap()
                .timestamp(),
        )
    }
}

fn read_process_cmd(frequency: f32) -> Result<BufReader<ChildStdout>, Error> {
    let stdout =  Command::new("sh")
        .arg("-c")
        .arg(format!("while true; do ps -eo pid,ppid,uid,lstart,pcpu,pmem,stat,args --no-headers && sleep {frequency}; done"))
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(||  Error::Stdout {
            msg: "Could not capture standard output.".to_string(),
        })?;
    Ok(BufReader::new(stdout))
}

pub async fn producer(
    sender: Sender<Process>,
    stop_flag: &Arc<AtomicBool>,
    frequency: f32,
) -> Result<(), Error> {
    let reader = read_process_cmd(frequency)?;
    let mut raw_processes = reader.lines().map_while(Result::ok);

    while !stop_flag.load(Ordering::Relaxed) {
        if let Some(raw_process) = raw_processes.next() {
            let process = Process::from(&raw_process)?;
            if let Err(e) = sender.send(process).await {
                return Err(Error::Channel(Box::new(e)));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, NaiveDateTime};

    #[test]
    fn test_process_from_valid_row() {
        let row = "1234 5678 1001 Sat Dec 14 18:39:49 2024 10.5 1.2 S /bin/bash";
        let expected_process = Process {
            pid: 1234,
            ppid: 5678,
            uid: 1001,
            lstart: NaiveDateTime::parse_from_str(
                "Sat Dec 14 18:39:49 2024",
                "%a %b %d %H:%M:%S %Y",
            )
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp(),
            pcpu: 10.5,
            pmem: 1.2,
            status: "S".to_string(),
            command: "/bin/bash".to_string(),
            created_at: 0,
        };

        let process = Process::from(row).unwrap();

        assert_eq!(process.pid, expected_process.pid);
        assert_eq!(process.ppid, expected_process.ppid);
        assert_eq!(process.uid, expected_process.uid);
        assert_eq!(process.pcpu, expected_process.pcpu);
        assert_eq!(process.pmem, expected_process.pmem);
        assert_eq!(process.status, expected_process.status);
        assert_eq!(process.command, expected_process.command);
        assert!((process.lstart - expected_process.lstart).abs() < 5);
    }

    #[test]
    fn test_parse_unix_ctime() {
        let date_chunks = ["Sat", "Dec", "14", "18:39:49", "2024"];

        let expected_timestamp =
            NaiveDateTime::parse_from_str("Sat Dec 14 18:39:49 2024", "%a %b %d %H:%M:%S %Y")
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp();

        let parsed_timestamp = Process::parse_unix_ctime(&date_chunks).unwrap();

        assert_eq!(parsed_timestamp, expected_timestamp);
    }
}
