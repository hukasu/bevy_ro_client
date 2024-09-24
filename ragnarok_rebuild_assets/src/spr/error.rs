use std::fmt::Display;

use crate::{common::Version, pal};

#[derive(Debug)]
pub enum Error {
    WrongSignature,
    UnsupportedVersion(Version),
    RLE,
    IncompleteRead(usize),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::WrongSignature => "Sprite had wrong signature.".to_owned(),
            Self::UnsupportedVersion(version) => {
                format!("Sprite had unsupported version {}.", version)
            }
            Self::RLE => {
                "Size of image after RLE decompression does not match image dimensions".to_owned()
            }
            Self::IncompleteRead(len) => {
                format!(
                    "Sprite finished loading but there was still {} bytes on the buffer.",
                    len
                )
            }
            Self::Io(io) => format!("An IO error occurred while reading a Sprite. '{io}'"),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for Error {}

impl From<pal::Error> for Error {
    fn from(value: pal::Error) -> Self {
        match value {
            pal::Error::Io(io) => Error::Io(io),
        }
    }
}
