pub mod error;

use crate::error::Error;
use regex::Captures;
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

const ROW_REGEX: &str = r"(\d+\.\d+)\s+(\S+)\s+(\S+)\[(\d+)\]:\s+(.+)";

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct JournalLog {
    pub timestamp: f64,
    pub hostname: String,
    pub command: String,
    pub pid: u32,
    pub message: String,
}

impl JournalLog {
    fn from(captures: Captures<'_>) -> Result<JournalLog, Error> {
        Ok(JournalLog {
            timestamp: captures[1].parse::<f64>()?,
            hostname: captures[2].to_string(),
            command: captures[3].to_string(),
            pid: captures[4].parse::<u32>()?,
            message: captures[5].to_string(),
        })
    }
}

fn read_journal_cmd() -> Result<BufReader<ChildStdout>, Error> {
    let stdout = Command::new("journalctl")
        .args(["-o", "short-unix", "-f"])
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::Stdout {
            msg: "Could not capture standard output.".to_string(),
        })?;
    Ok(BufReader::new(stdout))
}

pub async fn producer(
    sender: Sender<JournalLog>,
    stop_flag: &Arc<AtomicBool>,
) -> Result<(), Error> {
    let reader = read_journal_cmd()?;
    let regex = Regex::new(ROW_REGEX)?;
    let mut raw_logs = reader.lines().map_while(Result::ok);

    while !stop_flag.load(Ordering::Relaxed) {
        if let Some(raw_log) = raw_logs.next() {
            if let Some(captures) = regex.captures(&raw_log) {
                let log = JournalLog::from(captures)?;
                if let Err(e) = sender.send(log).await {
                    return Err(Error::Channel(Box::new(e)));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_log_from_captures() {
        let regex = Regex::new(ROW_REGEX).unwrap();
        let log_entry = "1734127260.194694 lpt01 sudo[48119]: pam_unix(sudo:session): session opened for user root(uid=0) by (uid=1000)";

        let captures = regex.captures(log_entry).expect("Regex should match");

        let journal_log =
            JournalLog::from(captures).expect("Should parse captures into JournalLog");

        assert_eq!(
            journal_log,
            JournalLog {
                timestamp: 1734127260.194694,
                hostname: "lpt01".to_string(),
                command: "sudo".to_string(),
                pid: 48119,
                message:
                    "pam_unix(sudo:session): session opened for user root(uid=0) by (uid=1000)"
                        .to_string(),
            }
        );
    }
}
