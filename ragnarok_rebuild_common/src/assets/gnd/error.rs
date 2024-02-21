use std::io;

use crate::assets::common::Version;

#[derive(Debug)]
pub enum Error {
    InvalidSignature(Box<str>),
    UnknownVersion(Version),
    Io(io::Error),
    IncompleteRead(Version, usize),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature(signature) => {
                write!(f, "GND had an invalid signature. '{signature}'")
            }
            Self::UnknownVersion(version) => {
                write!(f, "The GND version '{version}' is unknown.")
            }
            Self::Io(err) => write!(f, "Could not read GND due to IO error. '{err}'"),
            Self::IncompleteRead(version, unread) => write!(
                f,
                "Could not read GND to the end. GND V{version} had {unread} unread bytes."
            ),
        }
    }
}

impl std::error::Error for Error {}
