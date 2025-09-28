use std::fmt::Display;

use ragnarok_rebuild_common::Version;

#[derive(Debug)]
pub enum Error {
    InvalidSignature([u8; 4]),
    UnknownVersion(Version),
    IncompleteRead(Version, usize),
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature(signature) => {
                write!(
                    f,
                    "Gat had an invalid signature. '{}'",
                    signature.escape_ascii()
                )
            }
            Self::UnknownVersion(version) => {
                write!(f, "Gat version '{version}' is unknown.")
            }
            Self::IncompleteRead(version, unread) => write!(
                f,
                "Could not read Gat to the end. Gat V{version} had {unread} unread bytes."
            ),
            Self::Io(err) => write!(f, "Could not read Gat file due to Io error. '{}'", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
