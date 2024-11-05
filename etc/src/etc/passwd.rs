use crate::etc::error::Error;
use crate::etc::EtcReader;
use std::env::consts;
use std::process::Command;

const FILE_PATH: &str = "/etc/passwd";

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct User {
    pub name: String,
    pub uid: i16,
}

impl EtcReader<User> for User {
    fn read_etc_file(path: Option<&str>) -> Result<Vec<User>, Error> {
        match consts::OS {
            "macos" => get_user_macos(),
            "linux" => get_user_linux(path),
            _ => unimplemented!(),
        }
    }
}

// Linux

fn parse_cut_output(output: &str) -> Result<Vec<User>, Error> {
    output
        .lines()
        .filter(|row| !row.starts_with('#'))
        .map(|row| {
            let fields: Vec<&str> = row.split(':').collect();
            Ok(User {
                name: fields[0].to_string(),
                uid: fields[1].parse::<i16>()?,
            })
        })
        .collect()
}

fn get_user_linux(path: Option<&str>) -> Result<Vec<User>, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        FILE_PATH
    };
    let output = Command::new("cut").args(["-d:", "-f1,3", path]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_cut_output(&stdout)
}

// MacOS

fn parse_dscl_output(output: &str) -> Result<Vec<User>, Error> {
    output
        .lines()
        .map(|row| {
            let fields: Vec<&str> = row.split_whitespace().collect();
            Ok(User {
                name: fields[..fields.len() - 1].join(" ").to_string(),
                uid: fields.last().unwrap().parse::<i16>()?,
            })
        })
        .collect()
}

fn get_user_macos() -> Result<Vec<User>, Error> {
    let output = Command::new("dscl")
        .args([".", "-list", "/Users", "UniqueID"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_dscl_output(&stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_etc_file_success() {
        assert!(!User::read_etc_file(None).unwrap().is_empty());
    }

    #[test]
    fn test_parse_dscl_output_success() {
        let output = "user1 501\nuser2 502\n";
        let result = parse_dscl_output(output).unwrap();
        let expected = vec![
            User {
                name: "user1".to_string(),
                uid: 501,
            },
            User {
                name: "user2".to_string(),
                uid: 502,
            },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_dscl_with_space_name() {
        let output = "user one 10";
        let result = parse_dscl_output(output).unwrap();
        assert_eq!(
            result,
            vec![User {
                name: "user one".to_string(),
                uid: 10,
            }]
        );
    }

    #[test]
    fn test_parse_dscl_output_malformed_input() {
        let output = "user1\nuser2 502";
        let result = parse_dscl_output(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_dscl_output_empty_output() {
        let output = "";
        let result = parse_dscl_output(output).unwrap();
        let expected: Vec<User> = vec![];
        assert_eq!(result, expected);
    }
}
