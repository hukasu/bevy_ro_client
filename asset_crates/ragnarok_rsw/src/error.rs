use ragnarok_rebuild_common::Version;

#[derive(Debug)]
pub enum Error {
    InvalidSignature,
    UnknownVersion(Version),
    Io(std::io::Error),
    UnknownObjectType(u32),
    IncompleteRead(Version, usize),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Rsw file had wrong signature."),
            Self::UnknownVersion(version) => {
                write!(f, "The Rsw version '{version}' is unknown.")
            }
            Self::Io(err) => write!(f, "An IO error occurred while reading RSW. '{err}'"),
            Self::UnknownObjectType(obj_type) => {
                write!(f, "Rsw had an object of unknown type ({obj_type}).")
            }
            Self::IncompleteRead(version, unread) => write!(
                f,
                "Could not read Rsw to the end. Rsw v{version} had {unread} unread bytes."
            ),
        }
    }
}

impl std::error::Error for Error {}
