use crate::error::Error;
use chrono::Local;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

pub mod error;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct OpenFile {
    pub command: String,  // Command
    pub pid: u32,         // Process ID
    pub uid: i16,         // User ID
    pub fd: String,       // File Descriptor
    pub _type: String,    // Column type
    pub device: String,   // Device
    pub size: u64,        // Size
    pub node: String,     // Node
    pub name: String,     // Name
    pub _created_at: i64, // Timestamp command execution
}

impl OpenFile {
    pub fn new(pid: u32, uid: i16, command: &str) -> Self {
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
            _created_at: Local::now().timestamp_millis(),
        }
    }
}

fn parse_cmd_output(output: &str) -> Result<Vec<OpenFile>, Error> {
    let mut open_files: Vec<OpenFile> = vec![];
    let of_per_process: Vec<String> = split_of_per_process(output);
    for process in of_per_process {
        let rows_per_process: Vec<String> = split_process_per_rows(&process);
        let header = deserialize_header(&rows_per_process[0])?;
        for row in &rows_per_process[1..] {
            open_files.push(row_to_struct(&header, row))
        }
    }
    Ok(open_files)
}

fn split_of_per_process(output: &str) -> Vec<String> {
    if output.is_empty() {
        vec![]
    } else {
        output.split("\np").map(String::from).collect()
    }
}

fn split_process_per_rows(of_per_process: &str) -> Vec<String> {
    if of_per_process.is_empty() {
        vec![]
    } else {
        of_per_process.split("\nf").map(String::from).collect()
    }
}

fn deserialize_header(header: &str) -> Result<(u32, i16, String), Error> {
    let headers: Vec<&str> = header.lines().collect();
    let pid: u32 = headers[0].replace('p', "").parse()?;
    let uid: i16 = headers[2][1..].parse()?;
    let command: String = headers[1][1..].to_string();
    Ok((pid, uid, command))
}

fn row_to_struct(header: &(u32, i16, String), row: &str) -> OpenFile {
    let fields: Vec<&str> = row.lines().collect();
    let mut buffer_open_file: OpenFile = OpenFile::new(header.0, header.1, &header.2);
    buffer_open_file.fd = fields[0].to_string();
    for field in &fields[1..] {
        match &field[..1] {
            "t" => buffer_open_file._type = field[1..].to_string(),
            "s" => buffer_open_file.size = field[1..].parse().unwrap(),
            "i" => buffer_open_file.node = field[1..].to_string(),
            "D" => buffer_open_file.device = field[1..].to_string(),
            "n" => buffer_open_file.name = field[1..].to_string(),
            other => panic!("invalid lsof field label {}", other),
        }
    }
    buffer_open_file
}

fn lsof_cmd() -> Result<Output, Error> {
    Ok(Command::new("lsof")
        .args(["-F", "pcuftDsin", "/"])
        .output()?)
}

pub async fn producer(
    sender: Sender<OpenFile>,
    stop_flag: &Arc<AtomicBool>,
    frequency: u64,
) -> Result<(), Error> {
    while !stop_flag.load(Ordering::Relaxed) {
        let open_files = parse_cmd_output(&String::from_utf8_lossy(&lsof_cmd()?.stdout))?;
        for open_file in open_files {
            if let Err(e) = sender.send(open_file).await {
                return Err(Error::Channel(Box::new(e)));
            }
        }
        sleep(Duration::from_millis(frequency)).await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_lsof_output() -> String {
        "p163
cloginwindow
u501
fcwd
tDIR
D0x1000010
s640
i2
n/
ftxt
tREG
D0x1000010
s2722512
i1152921500312132720
n/System/Library/CoreServices/loginwindow.app/Contents/MacOS/loginwindow
p8015
cmdworker_shared
u501
fcwd
tDIR
D0x1000010
s640
i2
n/
ftxt
tREG
D0x1000010
s1133680
i1152921500312170301
n/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/Metadata.framework/Versions/A/Support/mdworker_shared
ftxt
tREG
D0x1000010
s58184
i11556174
n/Library/Preferences/Logging/.plist-cache.DCgGV34s
".to_string()
    }

    #[test]
    fn test_split_of_per_process() {
        let rows = split_of_per_process(&create_lsof_output());
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_split_of_per_process_with_empty() {
        let rows = split_of_per_process("");
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn test_split_process_per_rows() {
        let row = split_of_per_process(&create_lsof_output())[0].clone();
        let rows = split_process_per_rows(&row);
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn test_split_process_per_rows_with_empty() {
        let rows = split_process_per_rows("");
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn test_deserialize_header() {
        let row = split_of_per_process(&create_lsof_output())[0].clone();
        let row = split_process_per_rows(&row)[0].clone();
        let (pid, uid, command) = deserialize_header(&row).unwrap();
        assert_eq!(pid, 163);
        assert_eq!(uid, 501);
        assert_eq!(command, "loginwindow")
    }

    #[test]
    fn test_row_to_struct() {
        let row = split_of_per_process(&create_lsof_output())[0].clone();
        let header_row = split_process_per_rows(&row)[0].clone();
        let process_row = split_process_per_rows(&row)[1].clone();
        let (pid, uid, command) = deserialize_header(&header_row).unwrap();
        let open_file = row_to_struct(&(pid, uid, command), &process_row);

        assert_eq!(open_file.command, "loginwindow");
        assert_eq!(open_file.pid, 163);
        assert_eq!(open_file.uid, 501);
        assert_eq!(open_file.fd, "cwd");
        assert_eq!(open_file._type, "DIR");
        assert_eq!(open_file.device, "0x1000010");
        assert_eq!(open_file.size, 640);
        assert_eq!(open_file.node, "2");
        assert_eq!(open_file.name, "/");
    }
}
