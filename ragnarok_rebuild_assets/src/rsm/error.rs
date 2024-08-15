use std::io;

use crate::common::Version;

#[derive(Debug)]
pub enum Error {
    InvalidSignature(Box<str>),
    Io(io::Error),
    InvalidMeshName,
    InvalidShadeType(i32),
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
                write!(f, "RSM had an invalid signature. '{signature}'")
            }
            Self::Io(err) => write!(f, "Could not read RSM due to IO error. '{err}'"),
            Self::InvalidMeshName => {
                write!(f, "Failed to read the meshs name or mesh's parent's name.")
            }
            Self::InvalidShadeType(shade_type) => {
                write!(f, "RSM had invalid ShadeType '{shade_type}'.")
            }
            Self::IncompleteRead(version, unread) => write!(
                f,
                "Could not read RSM to the end. RSM V{version} had {unread} unread bytes."
            ),
        }
    }
}

impl std::error::Error for Error {}
