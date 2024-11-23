use crate::lsof::error::Error;
use crate::lsof::{FileType, Lsof, OpenFile};
use std::process::{Command, Output};

pub struct Unix;

impl Lsof for Unix {
    fn exec(file_type: &FileType) -> Result<Vec<OpenFile>, Error> {
        match file_type {
            FileType::REGULAR => Ok(Self::parse_output(&String::from_utf8_lossy(
                &Self::lsof_mount_file()?.stdout,
            ))?),
            FileType::NETWORK => Ok(Self::parse_output(&String::from_utf8_lossy(
                &Self::lsof_network()?.stdout,
            ))?),
            FileType::ALL => {
                let mut open_files =
                    Self::parse_output(&String::from_utf8_lossy(&Self::lsof_network()?.stdout))?;
                open_files.extend(Self::parse_output(&String::from_utf8_lossy(
                    &Self::lsof_mount_file()?.stdout,
                ))?);
                Ok(open_files)
            }
        }
    }
}

impl Unix {
    fn lsof_network() -> Result<Output, Error> {
        Ok(Command::new("lsof")
            .args(["-F", "pcuftDsin", "-i"])
            .output()?)
    }

    fn lsof_mount_file() -> Result<Output, Error> {
        Ok(Command::new("lsof")
            .args(["-F", "pcuftDsin", "/"])
            .output()?)
    }

    fn parse_output(output: &str) -> Result<Vec<OpenFile>, Error> {
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

#[cfg(test)]
mod tests {

    use crate::lsof::unix::{
        deserialize_header, row_to_struct, split_of_per_process, split_process_per_rows,
    };

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
