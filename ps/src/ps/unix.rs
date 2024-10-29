use crate::ps::error::Error;
use crate::ps::{Process, Ps};
use chrono::{Local, NaiveDateTime};
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
            created_at: Local::now().timestamp(),
        })
    }

    fn parse_date(date_chunks: &[&str]) -> Result<i64, Error> {
        let format = "%a %b %d %H:%M:%S %Y";
        Ok(
            NaiveDateTime::parse_from_str(date_chunks.join(" ").as_str(), format)?
                .and_local_timezone(Local)
                .unwrap()
                .timestamp(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ps::unix::Unix;
    use crate::ps::Ps;

    fn create_ps_output() -> String {
        "PID  PPID   UID                          STARTED %CPU %MEM STAT COMMAND
    1     0     0 Tue Aug 29 08:01:10 2023  0.1  0.3 Ss   /sbin/init
 1234     1  1000 Tue Aug 29 09:05:12 2023  0.0  1.2 S    /usr/lib/xorg/Xorg :0 -seat seat0 -auth /run/lightdm/root/:0 -nolisten tcp vt7 -novtswitch
 5678  1234  1000 Tue Aug 29 09:15:05 2023  0.2  0.5 R    /usr/bin/python3 /home/user/script.py
 9101  5678  1000 Tue Aug 29 10:00:02 2023  0.0  0.1 S    /bin/bash
".to_string()
    }

    #[test]
    fn test_parse_output() {
        let processes = Unix::parse_output(&create_ps_output()).unwrap();
        assert_eq!(processes.len(), 4);
        assert_eq!(processes.last().unwrap().pid, 9101);
        assert_eq!(processes[1].command, "/usr/lib/xorg/Xorg :0 -seat seat0 -auth /run/lightdm/root/:0 -nolisten tcp vt7 -novtswitch")
    }

    #[test]
    fn test_parse_row() {
        let row = "1234     1  1000 Tue Aug 29 09:05:12 2023  0.0  1.2 S    /usr/lib/xorg/Xorg :0 -seat seat0 -auth /run/lightdm/root/:0 -nolisten tcp vt7 -novtswitch";
        let process = Unix::parse_row(row).unwrap();
        assert_eq!(process.pid, 1234);
        assert_eq!(process.ppid, 1);
        assert_eq!(process.uid, 1000);
        assert_eq!(process.pcpu, 0.0);
        assert_eq!(process.pmem, 1.2);
        assert_eq!(process.status, "S");
        assert_eq!(process.command, "/usr/lib/xorg/Xorg :0 -seat seat0 -auth /run/lightdm/root/:0 -nolisten tcp vt7 -novtswitch");
    }
}
