use chrono;
use std::num;
use std::{fmt, io};

pub enum Error {
    ParseDate(chrono::ParseError),
    ParseInt(num::ParseIntError),
    ParseFloat(num::ParseFloatError),
    IO(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseDate(ref err) => write!(f, "{}", err),
            Error::ParseInt(ref err) => write!(f, "{}", err),
            Error::ParseFloat(ref err) => write!(f, "{}", err),
            Error::IO(ref err) => write!(f, "{}", err),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(value: chrono::ParseError) -> Self {
        Error::ParseDate(value)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(value: num::ParseIntError) -> Self {
        Error::ParseInt(value)
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(value: num::ParseFloatError) -> Self {
        Error::ParseFloat(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IO(value)
    }
}
