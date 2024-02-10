use super::version::Version;

#[derive(Debug)]
pub enum Error {
    InvalidSignature(Box<str>),
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
            Self::InvalidSignature(signature) => write!(f, "The signature {signature} is invalid."),
            Self::UnknownVersion(version) => {
                write!(f, "The version '{version}' is unknown.")
            }
            Self::Io(err) => write!(f, "An IO error occured while reading RSW. '{err}'"),
            Self::UnknownObjectType(obj_type) => {
                write!(f, "RSW had an object of unknown type ({obj_type}).")
            }
            Self::IncompleteRead(version, unread) => write!(
                f,
                "Could not read RSW to the end. RSW V{version} had {unread} unread bytes."
            ),
        }
    }
}

impl std::error::Error for Error {}
