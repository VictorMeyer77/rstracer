use crate::etc::error::Error;
use crate::etc::EtcReader;
use std::env::consts;
use std::process::Command;

const FILE_PATH: &str = "/etc/passwd";

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct User {
    pub name: String,
    pub uid: i32,
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
        .filter(|row| !row.starts_with('#') && !row.is_empty())
        .map(|row| {
            let fields: Vec<&str> = row.split(':').collect();
            Ok(User {
                name: fields[0].to_string(),
                uid: fields[1].parse::<i32>()?,
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
                uid: fields.last().unwrap().parse::<i32>()?,
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

    #[test]
    fn test_parse_cut_output_valid_data() {
        let output = "john:1000\nmary:1001\n# comment line\npaul:1002";
        let expected = vec![
            User {
                name: "john".to_string(),
                uid: 1000,
            },
            User {
                name: "mary".to_string(),
                uid: 1001,
            },
            User {
                name: "paul".to_string(),
                uid: 1002,
            },
        ];
        let result = parse_cut_output(output).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_cut_output_invalid_uid() {
        let output = "john:not_a_number\nmary:1001";
        let result = parse_cut_output(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cut_output_empty_lines_and_comments() {
        let output = "# this is a comment line\n\nmary:1001\n\n";
        let expected = vec![User {
            name: "mary".to_string(),
            uid: 1001,
        }];
        let result = parse_cut_output(output).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_cut_output_empty_input() {
        let output = "";
        let expected: Vec<User> = vec![];
        let result = parse_cut_output(output).unwrap();
        assert_eq!(result, expected);
    }
}
