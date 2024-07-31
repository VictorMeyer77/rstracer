use crate::lsof::error::Error;
use crate::lsof::{Lsof, OpenFile};
use log::warn;
use std::process::{Command, Output};

pub struct Unix;

impl Lsof for Unix {
    fn os_command() -> Result<Output, Error> {
        Ok(Command::new("lsof").args(["-F", "pcuftDsin"]).output()?)
    }

    fn parse_output(output: &str) -> Result<Vec<OpenFile>, Error> {
        let mut open_files: Vec<OpenFile> = vec![];
        let of_per_process: Vec<String> = split_of_per_process(output);
        for process in of_per_process {
            let rows_per_process: Vec<String> = split_process_per_rows(&process);
            let header = deserialize_header(&rows_per_process[0])?;
            for row in &rows_per_process[1..] {
                if let Ok(file) = row_to_struct(&header, row) {
                    open_files.push(file)
                } else {
                    warn!("Open File could not be parse {}", row)
                }
            }
        }
        Ok(open_files)
    }
}

fn split_of_per_process(output: &str) -> Vec<String> {
    output.split("\np").map(String::from).collect()
}

fn split_process_per_rows(of_per_process: &str) -> Vec<String> {
    of_per_process.split("\nf").map(String::from).collect()
}

fn deserialize_header(header: &str) -> Result<(u32, u32, String), Error> {
    let headers: Vec<&str> = header.lines().collect();
    let pid: u32 = headers[0].replace("p", "").parse()?;
    let uid: u32 = headers[2][1..].parse()?;
    let command: String = headers[1][1..].to_string();
    Ok((pid, uid, command))
}

fn row_to_struct(header: &(u32, u32, String), row: &str) -> Result<OpenFile, Error> {
    let fields: Vec<&str> = row.lines().collect();
    let mut invalid_row: bool = false;
    let mut device: String = "".to_string();
    let mut _type: String = "".to_string();
    let mut size: u32 = 0;
    let mut node: String = "".to_string();
    let mut name: String = "".to_string();
    fields[1..].iter().for_each(|field| match &field[..1] {
        "t" => _type = field[1..].to_string(),
        "s" => {
            if let Ok(s) = field[1..].parse() {
                size = s
            } else {
                invalid_row = true
            }
        }
        "i" => node = field[1..].to_string(),
        "D" => device = field[1..].to_string(),
        "n" => name = field[1..].to_string(),
        _ => invalid_row = true,
    });
    if invalid_row {
        Err(Error::ParseRow {
            row: row.to_string(),
        })
    } else {
        Ok(OpenFile {
            command: header.2.clone(),
            pid: header.0,
            uid: header.1,
            fd: fields[0].to_string(),
            _type,
            device,
            size,
            node,
            name,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::lsof::unix::Unix;
    use crate::lsof::Lsof;
    use std::env::consts;

    #[test]
    fn unix_integration_test() {
        if ["linux", "macos", "android", "ios"].contains(&consts::OS) {
            let files = Unix::exec().unwrap();
            assert!(files.len() > 10);
            assert_eq!(files.last().unwrap().command, "lsof")
        }
    }
}
