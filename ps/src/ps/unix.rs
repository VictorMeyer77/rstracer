use crate::ps::error::Error;
use crate::ps::{Process, Ps};
use chrono::NaiveDateTime;
use std::process::{Command, Output};

pub struct Unix;

impl Ps for Unix {
    fn os_command() -> Result<Output, Error> {
        Ok(Command::new("ps")
            .args(["-eo", "pid,ppid,uid,lstart,pcpu,pmem,stat,args"])
            .output()?)
    }

    fn parse_row(row: &str) -> Result<Process, Error> {
        let chunks: Vec<&str> = row.split_whitespace().collect();
        Ok(Process {
            pid: chunks[0].parse()?,
            ppid: chunks[1].parse()?,
            uid: chunks[2].parse()?,
            lstart: Self::parse_date(&chunks[3..8])?,
            pcpu: chunks[8].parse()?,
            pmem: chunks[9].parse()?,
            status: chunks[10].to_string(),
            command: chunks[11..].join(" "),
        })
    }

    fn parse_date(date_chunks: &[&str]) -> Result<i64, Error> {
        let format = "%a %b %d %H:%M:%S %Y";
        Ok(
            NaiveDateTime::parse_from_str(date_chunks.join(" ").as_str(), format)?
                .and_utc()
                .timestamp(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ps::unix::Unix;
    use crate::ps::Ps;
    use std::env::consts;

    #[test]
    fn unix_integration_test() {
        if ["linux", "macos", "android", "ios"].contains(&consts::OS) {
            let processes = Unix::exec().unwrap();
            assert!(processes.len() > 10);
            assert_eq!(
                processes.last().unwrap().command,
                "ps -eo pid,ppid,uid,lstart,pcpu,pmem,stat,args"
            )
        }
    }
}
