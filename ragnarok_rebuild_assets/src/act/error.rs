use std::fmt::Display;

use crate::common::Version;

#[derive(Debug)]
pub enum Error {
    WrongSignature,
    UnsupportedVersion(Version),
    IncompleteRead(usize),
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::WrongSignature => "Act file had wrong signature.".to_string(),
            Error::UnsupportedVersion(version) => {
                format!("Act file had unsupported version {}.", version)
            }
            Error::IncompleteRead(remainder) => {
                format!(
                    "Act file finished loading but still had {} bytes in stream.",
                    remainder
                )
            }
            Error::Io(io) => format!("An IO error occurred while reading 'act' file. '{io}'"),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}
